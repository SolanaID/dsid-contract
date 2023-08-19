#![cfg_attr(not(feature = "std"), no_std)]
use concordium_std::*;

#[derive(SchemaType, Serial, Reject, Debug, PartialEq)]
pub enum CustomError {
    /// Failed parsing the parameter.
    ParseParams,
    /// Failed logging: Log is full.
    LogFull,
    /// Failed logging: Log is malformed.
    LogMalformed,
    AccountsOnly,
    /// The token expiry is in the past.
    TokenExpired,
    /// The token has valid balances.
    TokenHasValidBalances,
}

/// Mapping the logging errors to ContractError.
impl From<LogError> for CustomError {
    fn from(le: LogError) -> Self {
        match le {
            LogError::Full => Self::LogFull,
            LogError::Malformed => Self::LogMalformed,
        }
    }
}

impl From<ParseError> for CustomError {
    fn from(_: ParseError) -> Self {
        Self::ParseParams
    }
}
