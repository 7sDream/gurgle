//! errors in gurgle command parsing and execution

use std::num::ParseIntError;

use thiserror::Error;

/// Can't parse string to any variant of target enum
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ParseEnumError;

/// Compile string to a gurgle command failed
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CompileError {
    /// Invalid syntax
    #[error("invalid gurgle syntax: {0}")]
    InvalidSyntax(String),
    /// Contains invalid number
    #[error("command contains invalid number")]
    ParseNumberError(#[from] ParseIntError),
    /// Dice roll or sided is negative
    #[error("Roll times or slides can't be negative")]
    DiceRollOrSidedNegative,
    /// Roll dice too much times
    #[error("dice roll times limit exceeded")]
    DiceRollTimesLimitExceeded,
    /// Dice have too many sides
    #[error("dice sides count limit exceeded")]
    DiceSidedCountLimitExceeded,
    /// Too many items in expression
    #[error("items count limit exceeded")]
    ItemCountLimitExceeded,
    /// Number item out of range
    #[error("number item out of range")]
    NumberItemOutOfRange,
}

impl<R: pest::RuleType> From<pest::error::Error<R>> for CompileError {
    fn from(err: pest::error::Error<R>) -> Self {
        Self::InvalidSyntax(format!("{}", err))
    }
}
