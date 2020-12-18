use std::fmt::Formatter;
use std::num::ParseIntError;

use crate::storage::errors::KVError;
use crate::storage::executor::command::CommandError;
use crate::storage::executor::predicate::PredicateError;
use crate::storage::executor::unit_content::UnitContentError;
use crate::storage::kvkey::KVKeyError;
use crate::utils::varint::{varint_decode, varint_encode, VarIntError};
use std::string::FromUtf8Error;

#[derive(Debug)]
pub enum ExecutorError {
    KVError(KVError),
    UnitContentError(UnitContentError),
    ParseIntError(ParseIntError),
    CommandError(CommandError),
    PredicateError(PredicateError),
    KVKeyError(KVKeyError),
    UnexpectedOutcome,
    ParseExecutorErrorToStringError,
}

#[derive(Debug)]
pub enum ExecutorErrorPrefix {
    KVError = 0x01,
    UnitContentError = 0x02,
    ParseIntError = 0x03,
    CommandError = 0x04,
    PredicateError = 0x05,
    KVKeyError = 0x06,
    UnexpectedOutcome = 0x07,
    ParseExecutorErrorToStringError = 0x08,
}

impl ExecutorError {
    pub fn marshal(&self) -> Vec<u8> {
        match self {
            ExecutorError::KVError(error) => {
                let mut result = vec![ExecutorErrorPrefix::KVError as u8];

                let error_bytes = error.to_string().as_bytes().to_vec();
                let error_bytes_length = error_bytes.len();
                let length_bytes = varint_encode(error_bytes_length as u64);

                result.extend_from_slice(&length_bytes);
                result.extend_from_slice(&error_bytes);
                return result;
            }
            ExecutorError::UnitContentError(error) => {
                let mut result = vec![ExecutorErrorPrefix::UnitContentError as u8];

                let error_bytes = error.to_string().as_bytes().to_vec();
                let error_bytes_length = error_bytes.len();
                let length_bytes = varint_encode(error_bytes_length as u64);

                result.extend_from_slice(&length_bytes);
                result.extend_from_slice(&error_bytes);
                return result;
            }
            ExecutorError::ParseIntError(error) => {
                let mut result = vec![ExecutorErrorPrefix::ParseIntError as u8];

                let error_bytes = error.to_string().as_bytes().to_vec();
                let error_bytes_length = error_bytes.len();
                let length_bytes = varint_encode(error_bytes_length as u64);

                result.extend_from_slice(&length_bytes);
                result.extend_from_slice(&error_bytes);
                return result;
            }
            ExecutorError::CommandError(error) => {
                let mut result = vec![ExecutorErrorPrefix::CommandError as u8];

                let error_bytes = error.to_string().as_bytes().to_vec();
                let error_bytes_length = error_bytes.len();
                let length_bytes = varint_encode(error_bytes_length as u64);

                result.extend_from_slice(&length_bytes);
                result.extend_from_slice(&error_bytes);
                return result;
            }
            ExecutorError::PredicateError(error) => {
                let mut result = vec![ExecutorErrorPrefix::PredicateError as u8];

                let error_bytes = error.to_string().as_bytes().to_vec();
                let error_bytes_length = error_bytes.len();
                let length_bytes = varint_encode(error_bytes_length as u64);

                result.extend_from_slice(&length_bytes);
                result.extend_from_slice(&error_bytes);
                return result;
            }
            ExecutorError::KVKeyError(error) => {
                let mut result = vec![ExecutorErrorPrefix::KVKeyError as u8];

                let error_bytes = error.to_string().as_bytes().to_vec();
                let error_bytes_length = error_bytes.len();
                let length_bytes = varint_encode(error_bytes_length as u64);

                result.extend_from_slice(&length_bytes);
                result.extend_from_slice(&error_bytes);
                return result;
            }
            ExecutorError::UnexpectedOutcome => vec![ExecutorErrorPrefix::UnexpectedOutcome as u8],
            ExecutorError::ParseExecutorErrorToStringError => {
                vec![ExecutorErrorPrefix::ParseExecutorErrorToStringError as u8]
            }
        }
    }

    pub fn parse_to_string(data: &[u8]) -> Result<(String, usize), ExecutorError> {
        let mut position = 0;
        let prefix = data[position];
        position += 1;

        if prefix == ExecutorErrorPrefix::UnexpectedOutcome as u8 {
            let error_string = "ExecutorError::UnexpectedOutcome".to_string();
            return Ok((error_string, position));
        } else if prefix == ExecutorErrorPrefix::ParseExecutorErrorToStringError as u8 {
            let error_string = "ExecutorErrorPrefix::ParseExecutorErrorToStringError".to_string();
            return Ok((error_string, position));
        } else {
            let (error_bytes_length, offset) = varint_decode(&data[position..])?;
            position += offset;

            let error_string_bytes =
                data[position..position + error_bytes_length as usize].to_vec();
            let error_string = String::from_utf8(error_string_bytes)?;
            position += error_bytes_length as usize;

            return Ok((error_string, position));
        }
    }
}

impl From<KVError> for ExecutorError {
    fn from(err: KVError) -> ExecutorError {
        ExecutorError::KVError(err)
    }
}

impl From<KVKeyError> for ExecutorError {
    fn from(err: KVKeyError) -> ExecutorError {
        ExecutorError::KVKeyError(err)
    }
}

impl From<UnitContentError> for ExecutorError {
    fn from(err: UnitContentError) -> ExecutorError {
        ExecutorError::UnitContentError(err)
    }
}

impl From<ParseIntError> for ExecutorError {
    fn from(err: ParseIntError) -> ExecutorError {
        ExecutorError::ParseIntError(err)
    }
}

impl From<CommandError> for ExecutorError {
    fn from(err: CommandError) -> ExecutorError {
        ExecutorError::CommandError(err)
    }
}

impl From<PredicateError> for ExecutorError {
    fn from(err: PredicateError) -> ExecutorError {
        ExecutorError::PredicateError(err)
    }
}

impl From<FromUtf8Error> for ExecutorError {
    fn from(_err: FromUtf8Error) -> ExecutorError {
        ExecutorError::ParseExecutorErrorToStringError
    }
}

impl From<VarIntError> for ExecutorError {
    fn from(_err: VarIntError) -> ExecutorError {
        ExecutorError::ParseExecutorErrorToStringError
    }
}

impl std::error::Error for ExecutorError {
    fn description(&self) -> &str {
        return "Executor error";
    }
}

impl std::fmt::Display for ExecutorError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        match self {
            ExecutorError::KVError(error) => {
                let error_str = format!("{}", error);
                write!(f, "{}::{}", "ExecutorError::KVError".to_string(), error_str)
            }
            ExecutorError::UnitContentError(error) => {
                let error_str = format!("{}", error);
                write!(
                    f,
                    "{}::{}",
                    "ExecutorError::UnitContentError".to_string(),
                    error_str
                )
            }
            ExecutorError::ParseIntError(error) => {
                let error_str = format!("{}", error);
                write!(
                    f,
                    "{}::{}",
                    "ExecutorError::ParseIntError".to_string(),
                    error_str
                )
            }
            ExecutorError::CommandError(error) => {
                let error_str = format!("{}", error);
                write!(
                    f,
                    "{}::{}",
                    "ExecutorError::CommandError".to_string(),
                    error_str
                )
            }
            ExecutorError::PredicateError(error) => {
                let error_str = format!("{}", error);
                write!(
                    f,
                    "{}::{}",
                    "ExecutorError::PredicateError".to_string(),
                    error_str
                )
            }
            ExecutorError::KVKeyError(error) => {
                let error_str = format!("{}", error);
                write!(
                    f,
                    "{}::{}",
                    "ExecutorError::PredicateError".to_string(),
                    error_str
                )
            }
            ExecutorError::UnexpectedOutcome => {
                write!(f, "{}", "ExecutorError::UnexpectedOutcome".to_string())
            }
            ExecutorError::ParseExecutorErrorToStringError => write!(
                f,
                "{}",
                "ExecutorError::ParseExecutorErrorToStringError".to_string()
            ),
        }
    }
}

pub type ExecutorResult<T> = Result<T, ExecutorError>;
