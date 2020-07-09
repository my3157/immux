use std::io::Error;
use std::num::ParseIntError;

use crate::storage::executor::errors::ExecutorError;

#[derive(Debug)]
pub enum ServerError {
    TinyHTTPError,
    ExecutorError(ExecutorError),
    BodyExtractionError(Error),
    UrlParsingError,
    BodyParsingError,
    ParseIntError(ParseIntError),
    HttpResponseError(Error),

    //    unimplemented error
    UnimplementedForGetGrouping,
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

pub type ServerResult<T> = Result<T, ServerError>;
