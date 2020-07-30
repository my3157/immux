use std::io::Error;
use std::num::ParseIntError;

use crate::storage::chain_height::ChainHeightError;
use crate::storage::instruction::InstructionError;
use crate::storage::transaction_manager::TransactionManagerError;

#[derive(Debug)]
pub enum KVError {
    IOError(Error),
    CommandError(InstructionError),
    RevertOutOfRange,
    ParseIntError(ParseIntError),
    ChainHeightError(ChainHeightError),
    PointToUnexpectedCommand,
    TransactionManagerError(TransactionManagerError),
}

impl From<Error> for KVError {
    fn from(err: Error) -> KVError {
        KVError::IOError(err)
    }
}

impl From<InstructionError> for KVError {
    fn from(err: InstructionError) -> KVError {
        KVError::CommandError(err)
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

pub type KVResult<T> = Result<T, KVError>;
