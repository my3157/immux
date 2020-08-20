use std::borrow::BorrowMut;
use std::collections::HashMap;

use crate::constants as Constants;
use crate::server::errors::{ServerError, ServerResult};
use crate::storage::chain_height::ChainHeight;
use crate::storage::executor::command::{Command, SelectCondition};
use crate::storage::executor::executor::Executor;
use crate::storage::executor::grouping_label::GroupingLabel;
use crate::storage::executor::outcome::Outcome;
use crate::storage::executor::unit_content::UnitContent;
use crate::storage::executor::unit_key::UnitKey;
use crate::storage::transaction_manager::TransactionId;

use crate::storage::executor::filter::parse_filter_string;
use tiny_http::{Method, Request, Response, Server};
use url::Url;

pub struct UrlInformation {
    pub queries: HashMap<String, String>,
    pub main_path: String,
}

impl UrlInformation {
    fn extract_numeric_query(&self, key: &str) -> ServerResult<u64> {
        match self.queries.get(key) {
            None => Err(ServerError::UrlParsingError),
            Some(string) => match string.parse::<u64>() {
                Err(_error) => Err(ServerError::UrlParsingError),
                Ok(value) => Ok(value),
            },
        }
    }
    fn extract_string_query(&self, key: &str) -> Option<String> {
        match self.queries.get(key) {
            None => None,
            Some(string) => Some(string.clone()),
        }
    }
}

pub fn run_server(mut executor: Executor, port: u16) -> ServerResult<()> {
    let address = format!("{}:{}", Constants::SERVER_END_POINT, port);
    match Server::http(address) {
        Ok(server) => {
            for mut request in server.incoming_requests() {
                let (status, body): (u16, String) =
                    match handle_request(request.borrow_mut(), &mut executor) {
                        Err(error) => (500, format!("Server error {:?}", error)),
                        Ok(outcome) => match outcome {
                            Outcome::Select(outcome) => {
                                let outcome_string_vec: Vec<String> = outcome
                                    .iter()
                                    .map(|unit_content| unit_content.to_string())
                                    .collect();
                                let body = outcome_string_vec.join("\r\n");
                                (200, body)
                            }
                            Outcome::InspectOne(outcome) => {
                                let mut body = String::new();
                                for (instruction, height) in outcome {
                                    body += &instruction.to_string();
                                    body += "\t";
                                    body += &format!("height: {:?}", height);
                                    body += "\r\n";
                                }
                                (200, body)
                            }
                            Outcome::InspectAll(outcome) => {
                                let mut body = String::new();
                                for (instruction, height) in outcome {
                                    body += &instruction.to_string();
                                    body += "\t";
                                    body += &format!("height: {:?}", height);
                                    body += "\r\n";
                                }
                                (200, body)
                            }
                            Outcome::CreateTransaction(transaction_id) => {
                                let body = transaction_id.as_u64().to_string();
                                (200, body)
                            }
                            _ => (200, String::from("Unspecified outcome")),
                        },
                    };

                let response = if body.is_empty() {
                    Response::from_string(UnitContent::Nil.to_string()).with_status_code(status)
                } else {
                    Response::from_string(body).with_status_code(status)
                };

                match request.respond(response) {
                    Ok(_) => {}
                    Err(error) => return Err(ServerError::HttpResponseError(error)),
                }
            }
        }
        Err(_error) => return Err(ServerError::TinyHTTPError),
    }
    return Ok(());
}

fn handle_request(request: &mut Request, executor: &mut Executor) -> ServerResult<Outcome> {
    let instruction = parse_http_request(request)?;

    match instruction {
        Command::Select {
            grouping,
            condition,
        } => {
            let content_vec = executor.get(&grouping, &condition)?;
            return Ok(Outcome::Select(content_vec));
        }
        Command::Insert {
            grouping,
            key,
            content,
        } => {
            executor.set(&grouping, &key, &content, None)?;
            return Ok(Outcome::InsertSuccess);
        }
        Command::RemoveOne { grouping, key } => {
            executor.remove_one(&grouping, &key, None)?;
            return Ok(Outcome::RemoveOneSuccess);
        }
        Command::RemoveAll => {
            executor.remove_all()?;
            return Ok(Outcome::RemoveAllSuccess);
        }
        Command::RevertOne {
            grouping,
            key,
            height,
        } => {
            executor.revert_one(&grouping, &key, &height, None)?;
            return Ok(Outcome::RevertOneSuccess);
        }
        Command::RevertAll { height } => {
            executor.revert_all(&height)?;
            return Ok(Outcome::RevertAllSuccess);
        }
        Command::InspectOne { grouping, key } => {
            let result = executor.inspect_one(&grouping, &key)?;
            return Ok(Outcome::InspectOne(result));
        }
        Command::InspectAll => {
            let result = executor.inspect_all()?;
            return Ok(Outcome::InspectAll(result));
        }
        Command::CreateTransaction => {
            let transaction_id = executor.start_transaction()?;
            return Ok(Outcome::CreateTransaction(transaction_id));
        }
        Command::TransactionCommit { transaction_id } => {
            executor.commit_transaction(transaction_id)?;
            return Ok(Outcome::TransactionCommitSuccess);
        }
        Command::TransactionAbort { transaction_id } => {
            executor.abort_transaction(transaction_id)?;
            return Ok(Outcome::TransactionAbortSuccess);
        }
        Command::TransactionalInsert {
            grouping,
            key,
            content,
            transaction_id,
        } => {
            executor.set(&grouping, &key, &content, Some(transaction_id))?;
            return Ok(Outcome::TransactionalInsertSuccess);
        }
        Command::TransactionalRemoveOne {
            grouping,
            key,
            transaction_id,
        } => {
            executor.remove_one(&grouping, &key, Some(transaction_id))?;
            return Ok(Outcome::TransactionalRemoveOneSuccess);
        }
        Command::TransactionalRevertOne {
            grouping,
            key,
            height,
            transaction_id,
        } => {
            executor.revert_one(&grouping, &key, &height, Some(transaction_id))?;
            return Ok(Outcome::TransactionalRevertOneSuccess);
        }
    }
}

fn parse_http_request(request: &mut Request) -> ServerResult<Command> {
    let mut incoming_body = String::new();
    match request.as_reader().read_to_string(&mut incoming_body) {
        Ok(_) => (),
        Err(error) => return Err(ServerError::BodyExtractionError(error)),
    }

    let url_info = parse_path(&request.url())?;
    let segments: Vec<&str> = url_info.main_path.split("/").collect();

    match request.method() {
        Method::Get => {
            if segments.len() >= 5 {
                let url_transactions_key_word = segments[1];
                let transaction_id_str = segments[2];
                let grouping_str = segments[3];
                let unit_key_str = segments[4];

                if url_transactions_key_word != Constants::URL_TRANSACTIONS_KEY_WORD
                    || unit_key_str.is_empty()
                {
                    return Err(ServerError::UrlParsingError);
                }

                let transaction_id = transaction_id_str.parse::<u64>()?;
                let transaction_id = TransactionId::new(transaction_id);
                let grouping = GroupingLabel::new(grouping_str.as_bytes());
                let unit_key = UnitKey::from(unit_key_str);

                let condition = SelectCondition::Key(unit_key, Some(transaction_id));
                let instruction = Command::Select {
                    grouping,
                    condition,
                };

                return Ok(instruction);
            } else if segments.len() >= 4 {
                let grouping_str = segments[1];
                let unit_key_str = segments[2];
                let url_journal_key_word = segments[3];

                if url_journal_key_word != Constants::URL_JOURNAL_KEY_WORD
                    || unit_key_str.is_empty()
                {
                    return Err(ServerError::UrlParsingError);
                }

                let grouping = GroupingLabel::new(grouping_str.as_bytes());
                let unit_key = UnitKey::from(unit_key_str);
                let instruction = Command::InspectOne {
                    grouping,
                    key: unit_key,
                };
                return Ok(instruction);
            } else if segments.len() >= 3 {
                let grouping_str = segments[1];
                let unit_key_str = segments[2];
                let grouping = GroupingLabel::new(grouping_str.as_bytes());
                let unit_key = UnitKey::from(unit_key_str);

                let condition = SelectCondition::Key(unit_key, None);
                let instruction = Command::Select {
                    grouping,
                    condition,
                };

                return Ok(instruction);
            } else if segments.len() >= 2 {
                if segments[1] == Constants::URL_JOURNAL_KEY_WORD {
                    let instruction = Command::InspectAll;
                    return Ok(instruction);
                } else {
                    if let Some(filter_string) =
                        url_info.extract_string_query(Constants::FILTER_KEY_WORD)
                    {
                        let grouping_str = segments[1];
                        let grouping = GroupingLabel::new(grouping_str.as_bytes());
                        let filter = parse_filter_string(filter_string)?;
                        let condition = SelectCondition::Filter(filter);

                        let instruction = Command::Select {
                            grouping,
                            condition,
                        };

                        return Ok(instruction);
                    } else {
                        return Err(ServerError::UnimplementedForGetGrouping);
                    }
                }
            } else {
                return Err(ServerError::UrlParsingError);
            }
        }
        Method::Put => {
            if segments.len() >= 5 {
                let url_transactions_key_word = segments[1];
                let transaction_id_str = segments[2];
                let grouping_str = segments[3];
                let unit_key_str = segments[4];

                if url_transactions_key_word != Constants::URL_TRANSACTIONS_KEY_WORD
                    || unit_key_str.is_empty()
                {
                    return Err(ServerError::UrlParsingError);
                }

                let grouping = GroupingLabel::new(grouping_str.as_bytes());
                let unit_key = UnitKey::from(unit_key_str);
                let transaction_id = transaction_id_str.parse::<u64>()?;

                if let Ok(height) = url_info.extract_numeric_query(Constants::HEIGHT) {
                    let height = ChainHeight::new(height);
                    let transaction_id = TransactionId::new(transaction_id);
                    let instruction = Command::TransactionalRevertOne {
                        grouping,
                        key: unit_key,
                        height,
                        transaction_id,
                    };
                    return Ok(instruction);
                } else {
                    let content = UnitContent::from(incoming_body.as_str());
                    let transaction_id = TransactionId::new(transaction_id);
                    let instruction = Command::TransactionalInsert {
                        grouping,
                        key: unit_key,
                        content,
                        transaction_id,
                    };
                    return Ok(instruction);
                }
            } else if segments.len() >= 3 {
                let grouping_str = segments[1];
                let unit_key_str = segments[2];

                if unit_key_str.is_empty() {
                    return Err(ServerError::UrlParsingError);
                }

                let grouping = GroupingLabel::new(grouping_str.as_bytes());
                let unit_key = UnitKey::from(unit_key_str);

                if let Ok(height) = url_info.extract_numeric_query(Constants::HEIGHT) {
                    let height = ChainHeight::new(height);
                    let instruction = Command::RevertOne {
                        grouping,
                        key: unit_key,
                        height,
                    };
                    return Ok(instruction);
                } else {
                    let content = UnitContent::from(incoming_body.as_str());
                    let instruction = Command::Insert {
                        grouping,
                        key: unit_key,
                        content,
                    };
                    return Ok(instruction);
                }
            } else if let Ok(height) = url_info.extract_numeric_query(Constants::HEIGHT) {
                let height = ChainHeight::new(height);
                let instruction = Command::RevertAll { height };
                return Ok(instruction);
            } else {
                return Err(ServerError::UrlParsingError);
            }
        }
        Method::Post => {
            let (url_transactions_key_word, transaction_id_str) = if segments.len() >= 3 {
                (segments[1], segments[2])
            } else if segments.len() == 2 {
                (segments[1], "")
            } else {
                ("", "")
            };

            if url_transactions_key_word != Constants::URL_TRANSACTIONS_KEY_WORD {
                return Err(ServerError::UrlParsingError);
            }

            if let Some(_) = url_info.extract_string_query(Constants::COMMIT_TRANSACTION_KEY_WORD) {
                if transaction_id_str.is_empty() {
                    return Err(ServerError::UrlParsingError);
                }

                let transaction_id = transaction_id_str.parse::<u64>()?;
                let transaction_id = TransactionId::new(transaction_id);

                let instruction = Command::TransactionCommit { transaction_id };
                return Ok(instruction);
            } else if let Some(_) =
                url_info.extract_string_query(Constants::ABORT_TRANSACTION_KEY_WORD)
            {
                if transaction_id_str.is_empty() {
                    return Err(ServerError::UrlParsingError);
                }

                let transaction_id = transaction_id_str.parse::<u64>()?;
                let transaction_id = TransactionId::new(transaction_id);

                let instruction = Command::TransactionAbort { transaction_id };
                return Ok(instruction);
            } else {
                let instruction = Command::CreateTransaction;
                return Ok(instruction);
            }
        }
        Method::Delete => {
            if segments.len() >= 5 {
                let url_transactions_key_word = segments[1];
                let transaction_id_str = segments[2];
                let grouping_str = segments[3];
                let unit_key_str = segments[4];

                if unit_key_str.is_empty()
                    || transaction_id_str.is_empty()
                    || url_transactions_key_word != Constants::URL_TRANSACTIONS_KEY_WORD
                {
                    return Err(ServerError::UrlParsingError);
                }

                let transaction_id = transaction_id_str.parse::<u64>()?;
                let transaction_id = TransactionId::new(transaction_id);
                let grouping = GroupingLabel::new(grouping_str.as_bytes());
                let unit_key = UnitKey::from(unit_key_str);

                let instruction = Command::TransactionalRemoveOne {
                    grouping,
                    key: unit_key,
                    transaction_id,
                };
                return Ok(instruction);
            } else if segments.len() >= 3 {
                let grouping_str = segments[1];
                let unit_key_str = segments[2];
                let grouping = GroupingLabel::new(grouping_str.as_bytes());
                let unit_key = UnitKey::from(unit_key_str);

                let instruction = Command::RemoveOne {
                    grouping,
                    key: unit_key,
                };
                return Ok(instruction);
            } else {
                let instruction = Command::RemoveAll;
                return Ok(instruction);
            }
        }
        _ => return Err(ServerError::BodyParsingError),
    }
}

pub fn parse_path(path: &str) -> ServerResult<UrlInformation> {
    let path_to_parse = format!("{}{}", "http://127.0.0.1", path);
    match Url::parse(&path_to_parse) {
        Err(_error) => Err(ServerError::UrlParsingError),
        Ok(parse) => {
            let url_queries: HashMap<_, _> = parse.query_pairs().into_owned().collect();
            Ok(UrlInformation {
                queries: url_queries,
                main_path: String::from(parse.path()),
            })
        }
    }
}
