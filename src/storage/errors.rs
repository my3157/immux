use std::fmt;
use std::io::Error;
use std::num::ParseIntError;

use crate::storage::chain_height::ChainHeightError;
use crate::storage::instruction::InstructionError;
use crate::storage::log_version::LogVersionError;
use crate::storage::transaction_manager::TransactionManagerError;

#[derive(Debug)]
pub enum KVError {
    IOError(Error),
    InstructionError(InstructionError),
    RevertOutOfRange,
    ParseIntError(ParseIntError),
    ChainHeightError(ChainHeightError),
    PointToUnexpectedInstruction,
    TransactionManagerError(TransactionManagerError),
    LogVersionError(LogVersionError),
}

impl fmt::Display for KVError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KVError::IOError(error) => {
                let error_string = error.to_string();
                write!(
                    f,
                    "{}::{}",
                    "VarIntError::UnexpectedFormat".to_string(),
                    error_string
                )
            }
            KVError::InstructionError(error) => {
                let error_string = error.to_string();
                write!(
                    f,
                    "{}::{}",
                    "KVError::InstructionError".to_string(),
                    error_string
                )
            }
            KVError::RevertOutOfRange => write!(f, "{}", "KVError::RevertOutOfRange".to_string()),
            KVError::ParseIntError(error) => {
                let error_string = error.to_string();
                write!(
                    f,
                    "{}::{}",
                    "KVError::ParseIntError".to_string(),
                    error_string
                )
            }
            KVError::ChainHeightError(error) => {
                let error_string = error.to_string();
                write!(
                    f,
                    "{}::{}",
                    "KVError::ChainHeightError".to_string(),
                    error_string
                )
            }
            KVError::PointToUnexpectedInstruction => {
                write!(f, "{}", "KVError::PointToUnexpectedInstruction".to_string())
            }
            KVError::TransactionManagerError(error) => {
                let error_string = error.to_string();
                write!(
                    f,
                    "{}::{}",
                    "KVError::TransactionManagerError".to_string(),
                    error_string
                )
            }
            KVError::LogVersionError(error) => {
                let error_string = error.to_string();
                write!(
                    f,
                    "{}::{}",
                    "KVError::LogVersionError".to_string(),
                    error_string
                )
            }
        }
    }
}

impl From<Error> for KVError {
    fn from(err: Error) -> KVError {
        KVError::IOError(err)
    }
}

impl From<InstructionError> for KVError {
    fn from(err: InstructionError) -> KVError {
        KVError::InstructionError(err)
    }
}

impl From<ParseIntError> for KVError {
    fn from(err: ParseIntError) -> KVError {
        KVError::ParseIntError(err)
    }
}

impl From<ChainHeightError> for KVError {
    fn from(err: ChainHeightError) -> KVError {
        KVError::ChainHeightError(err)
    }
}

impl From<TransactionManagerError> for KVError {
    fn from(err: TransactionManagerError) -> KVError {
        KVError::TransactionManagerError(err)
    }
}

impl From<LogVersionError> for KVError {
    fn from(err: LogVersionError) -> KVError {
        KVError::LogVersionError(err)
    }
}

pub type KVResult<T> = Result<T, KVError>;
