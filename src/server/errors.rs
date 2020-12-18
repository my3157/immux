use std::fmt;
use std::io::Error;
use std::num::ParseIntError;
use std::string::FromUtf8Error;
use std::sync::mpsc::{RecvError, SendError};

use crate::storage::executor::command::CommandError;
use crate::storage::executor::errors::ExecutorError;
use crate::storage::executor::predicate::PredicateError;
use crate::utils::varint::{varint_decode, varint_encode, VarIntError};

#[derive(Debug)]
pub enum ServerError {
    TinyHTTPError,
    ExecutorError(ExecutorError),
    BodyExtractionError(Error),
    UrlParsingError,
    BodyParsingError,
    ParseIntError(ParseIntError),
    HttpResponseError(Error),
    PredicateError(PredicateError),
    SenderError,
    ReceiverError(RecvError),
    TCPServerError(Error),
    CommandError(CommandError),
    ThreadError,
    ParseServerErrorToStringError,
}

#[derive(Debug)]
pub enum ServerErrorPrefix {
    TinyHTTPError = 0x01,
    ExecutorError = 0x02,
    BodyExtractionError = 0x03,
    UrlParsingError = 0x04,
    BodyParsingError = 0x05,
    ParseIntError = 0x06,
    HttpResponseError = 0x07,
    PredicateError = 0x08,
    SenderError = 0x09,
    ReceiverError = 0x0A,
    TCPServerError = 0x0B,
    CommandError = 0x0C,
    ThreadError = 0x0D,
    ParseServerErrorToStringError = 0x0E,
}

impl ServerError {
    pub fn marshal(&self) -> Vec<u8> {
        match self {
            ServerError::TinyHTTPError => vec![ServerErrorPrefix::TinyHTTPError as u8],
            ServerError::ExecutorError(executor_error) => {
                let mut result = vec![ServerErrorPrefix::ExecutorError as u8];
                let error_bytes = executor_error.marshal();
                result.extend_from_slice(&error_bytes);
                return result;
            }
            ServerError::BodyExtractionError(error) => {
                let mut result = vec![ServerErrorPrefix::BodyExtractionError as u8];
                let error_bytes = format!("{}", error).as_bytes().to_vec();
                let error_bytes_length = error_bytes.len();
                let length_bytes = varint_encode(error_bytes_length as u64);

                result.extend_from_slice(&length_bytes);
                result.extend_from_slice(&error_bytes);
                return result;
            }
            ServerError::UrlParsingError => vec![ServerErrorPrefix::UrlParsingError as u8],
            ServerError::BodyParsingError => vec![ServerErrorPrefix::BodyParsingError as u8],
            ServerError::ParseIntError(error) => {
                let mut result = vec![ServerErrorPrefix::ParseIntError as u8];
                let error_bytes = format!("{}", error).as_bytes().to_vec();
                let error_bytes_length = error_bytes.len();
                let length_bytes = varint_encode(error_bytes_length as u64);

                result.extend_from_slice(&length_bytes);
                result.extend_from_slice(&error_bytes);
                return result;
            }
            ServerError::HttpResponseError(error) => {
                let mut result = vec![ServerErrorPrefix::HttpResponseError as u8];
                let error_bytes = format!("{}", error).as_bytes().to_vec();
                let error_bytes_length = error_bytes.len();
                let length_bytes = varint_encode(error_bytes_length as u64);

                result.extend_from_slice(&length_bytes);
                result.extend_from_slice(&error_bytes);
                return result;
            }
            ServerError::PredicateError(predicate_error) => {
                let mut result = vec![ServerErrorPrefix::PredicateError as u8];
                let error_bytes = predicate_error.marshal();
                result.extend_from_slice(&error_bytes);
                return result;
            }
            ServerError::SenderError => vec![ServerErrorPrefix::SenderError as u8],
            ServerError::ReceiverError(error) => {
                let mut result = vec![ServerErrorPrefix::ReceiverError as u8];
                let error_bytes = format!("{}", error).as_bytes().to_vec();
                let error_bytes_length = error_bytes.len();
                let length_bytes = varint_encode(error_bytes_length as u64);

                result.extend_from_slice(&length_bytes);
                result.extend_from_slice(&error_bytes);
                return result;
            }
            ServerError::TCPServerError(error) => {
                let mut result = vec![ServerErrorPrefix::TCPServerError as u8];
                let error_bytes = format!("{}", error).as_bytes().to_vec();
                let error_bytes_length = error_bytes.len();
                let length_bytes = varint_encode(error_bytes_length as u64);

                result.extend_from_slice(&length_bytes);
                result.extend_from_slice(&error_bytes);
                return result;
            }
            ServerError::CommandError(error) => {
                let mut result = vec![ServerErrorPrefix::CommandError as u8];
                let error_bytes = error.marshal();
                result.extend_from_slice(&error_bytes);
                return result;
            }
            ServerError::ThreadError => vec![ServerErrorPrefix::ThreadError as u8],
            ServerError::ParseServerErrorToStringError => {
                vec![ServerErrorPrefix::ParseServerErrorToStringError as u8]
            }
        }
    }

    pub fn parse_to_string(data: &[u8]) -> Result<(String, usize), ServerError> {
        let mut position = 0;
        let prefix = data[position];
        position += 1;

        if prefix == ServerErrorPrefix::TinyHTTPError as u8 {
            let error_string = format!("{}", ServerError::TinyHTTPError);
            return Ok((error_string, position));
        } else if prefix == ServerErrorPrefix::ExecutorError as u8 {
            let (error_string, offset) = ExecutorError::parse_to_string(&data[position..])?;
            position += offset;
            return Ok((error_string, position));
        } else if prefix == ServerErrorPrefix::BodyExtractionError as u8 {
            let (error_bytes_length, offset) = varint_decode(&data[position..])?;
            position += offset;

            let error_string_bytes =
                data[position..position + error_bytes_length as usize].to_vec();
            let error_string = String::from_utf8(error_string_bytes)?;
            position += error_bytes_length as usize;

            return Ok((error_string, position));
        } else if prefix == ServerErrorPrefix::UrlParsingError as u8 {
            let error_string = format!("{}", ServerError::UrlParsingError);
            return Ok((error_string, position));
        } else if prefix == ServerErrorPrefix::BodyParsingError as u8 {
            let error_string = format!("{}", ServerError::BodyParsingError);
            return Ok((error_string, position));
        } else if prefix == ServerErrorPrefix::ParseIntError as u8 {
            let (error_bytes_length, offset) = varint_decode(&data[position..])?;
            position += offset;

            let error_string_bytes =
                data[position..position + error_bytes_length as usize].to_vec();
            let error_string = String::from_utf8(error_string_bytes)?;
            position += error_bytes_length as usize;

            return Ok((error_string, position));
        } else if prefix == ServerErrorPrefix::HttpResponseError as u8 {
            let (error_bytes_length, offset) = varint_decode(&data[position..])?;
            position += offset;

            let error_string_bytes =
                data[position..position + error_bytes_length as usize].to_vec();
            let error_string = String::from_utf8(error_string_bytes)?;
            position += error_bytes_length as usize;

            return Ok((error_string, position));
        } else if prefix == ServerErrorPrefix::PredicateError as u8 {
            let (error, offset) = PredicateError::parse_to_string(&data[position..])?;
            position += offset;
            return Ok((error, position));
        } else if prefix == ServerErrorPrefix::SenderError as u8 {
            let error_string = format!("{}", ServerError::SenderError);
            return Ok((error_string, position));
        } else if prefix == ServerErrorPrefix::ReceiverError as u8 {
            let (error_bytes_length, offset) = varint_decode(&data[position..])?;
            position += offset;

            let error_string_bytes =
                data[position..position + error_bytes_length as usize].to_vec();
            let error_string = String::from_utf8(error_string_bytes)?;
            position += error_bytes_length as usize;

            return Ok((error_string, position));
        } else if prefix == ServerErrorPrefix::TCPServerError as u8 {
            let (error_bytes_length, offset) = varint_decode(&data[position..])?;
            position += offset;

            let error_string_bytes =
                data[position..position + error_bytes_length as usize].to_vec();
            let error_string = String::from_utf8(error_string_bytes)?;
            position += error_bytes_length as usize;

            return Ok((error_string, position));
        } else if prefix == ServerErrorPrefix::CommandError as u8 {
            let (error_string, offset) = CommandError::parse_to_string(&data[position..])?;
            position += offset;
            return Ok((error_string, position));
        } else if prefix == ServerErrorPrefix::ThreadError as u8 {
            let error_string = format!("{}", ServerError::ThreadError);
            return Ok((error_string, position));
        } else {
            let error_string = format!("{}", ServerError::ParseServerErrorToStringError);
            return Ok((error_string, position));
        }
    }
}

impl From<Error> for ServerError {
    fn from(err: Error) -> ServerError {
        ServerError::TCPServerError(err)
    }
}

impl From<RecvError> for ServerError {
    fn from(err: RecvError) -> ServerError {
        ServerError::ReceiverError(err)
    }
}

impl<T> From<SendError<T>> for ServerError {
    fn from(_err: SendError<T>) -> ServerError {
        ServerError::SenderError
    }
}

impl From<ExecutorError> for ServerError {
    fn from(err: ExecutorError) -> ServerError {
        ServerError::ExecutorError(err)
    }
}

impl From<ParseIntError> for ServerError {
    fn from(err: ParseIntError) -> ServerError {
        ServerError::ParseIntError(err)
    }
}

impl From<CommandError> for ServerError {
    fn from(err: CommandError) -> ServerError {
        ServerError::CommandError(err)
    }
}

impl From<PredicateError> for ServerError {
    fn from(err: PredicateError) -> ServerError {
        ServerError::PredicateError(err)
    }
}

impl From<FromUtf8Error> for ServerError {
    fn from(_err: FromUtf8Error) -> ServerError {
        ServerError::ParseServerErrorToStringError
    }
}

impl From<VarIntError> for ServerError {
    fn from(_err: VarIntError) -> ServerError {
        ServerError::ParseServerErrorToStringError
    }
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServerError::TinyHTTPError => write!(f, "{}", "ServerError::TinyHTTPError".to_string()),
            ServerError::ExecutorError(executor_error) => {
                let executor_error_str = format!("{}", executor_error);
                write!(
                    f,
                    "{}::{}",
                    "ServerError::ExecutorError".to_string(),
                    executor_error_str
                )
            }
            ServerError::BodyExtractionError(error) => {
                let error_str = error.to_string();
                write!(
                    f,
                    "{}::{}",
                    "ServerError::ExecutorError".to_string(),
                    error_str
                )
            }
            ServerError::UrlParsingError => {
                write!(f, "{}", "ServerError::UrlParsingError".to_string())
            }
            ServerError::BodyParsingError => {
                write!(f, "{}", "ServerError::BodyParsingError".to_string())
            }
            ServerError::ParseIntError(error) => {
                let error_str = error.to_string();
                write!(
                    f,
                    "{}::{}",
                    "ServerError::ExecutorError".to_string(),
                    error_str
                )
            }
            ServerError::HttpResponseError(error) => {
                let error_str = error.to_string();
                write!(
                    f,
                    "{}::{}",
                    "ServerError::ExecutorError".to_string(),
                    error_str
                )
            }
            ServerError::PredicateError(predicate_error) => {
                let predicate_error_str = format!("{}", predicate_error);
                write!(
                    f,
                    "{}::{}",
                    "ServerError::ExecutorError".to_string(),
                    predicate_error_str
                )
            }
            ServerError::SenderError => write!(f, "{}", "ServerError::SenderError".to_string()),
            ServerError::ReceiverError(error) => {
                let error_str = error.to_string();
                write!(
                    f,
                    "{}::{}",
                    "ServerError::ExecutorError".to_string(),
                    error_str
                )
            }
            ServerError::TCPServerError(error) => {
                let error_str = error.to_string();
                write!(
                    f,
                    "{}::{}",
                    "ServerError::ExecutorError".to_string(),
                    error_str
                )
            }
            ServerError::CommandError(error) => {
                let command_error_str = format!("{}", error);
                write!(
                    f,
                    "{}::{}",
                    "ServerError::ExecutorError".to_string(),
                    command_error_str
                )
            }
            ServerError::ThreadError => write!(f, "{}", "ServerError::ThreadError".to_string()),
            ServerError::ParseServerErrorToStringError => write!(
                f,
                "{}",
                "ServerError::ParseServerErrorToStringError".to_string()
            ),
        }
    }
}

pub type ServerResult<S> = Result<S, ServerError>;
