use std::fmt::Formatter;
use std::num::ParseIntError;

use crate::storage::errors::KVError;
use crate::storage::executor::command::CommandError;
use crate::storage::executor::predicate::PredicateError;
use crate::storage::executor::unit_content::UnitContentError;
use crate::storage::kvkey::KVKeyError;

#[derive(Debug)]
pub enum ExecutorError {
    KVError(KVError),
    UnitContentError(UnitContentError),
    ParseIntError(ParseIntError),
    CommandError(CommandError),
    PredicateError(PredicateError),
    KVKeyError(KVKeyError),
    UnexpectedOutcome,
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
            },
            ExecutorError::UnitContentError(error) => unimplemented!(),
            ExecutorError::ParseIntError(error) => unimplemented!(),
            ExecutorError::CommandError(error) => unimplemented!(),
            ExecutorError::PredicateError(error) => unimplemented!(),
            ExecutorError::KVKeyError(error) => unimplemented!(),
            ExecutorError::UnexpectedOutcome => unimplemented!(),
        }
    }

    pub fn parse(data: &[u8]) -> Result<(Self, usize), ExecutorError> {
        unimplemented!()
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

impl std::error::Error for ExecutorError {
    fn description(&self) -> &str {
        return "Executor error";
    }
}

impl std::fmt::Display for ExecutorError {
    fn fmt(&self, _f: &mut Formatter) -> Result<(), std::fmt::Error> {
        unimplemented!();
    }
}

pub type ExecutorResult<T> = Result<T, ExecutorError>;
