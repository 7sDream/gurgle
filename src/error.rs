//! Errors when parse gurgle command

use std::num::ParseIntError;

use thiserror::Error;

/// Can't parse string to any variant of target enum.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ParseEnumError;

/// Parse string to as a gurgle command failed
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum GurgleError {
    /// Invalid syntax
    #[error("invalid gurgle syntax: {0}")]
    InvalidSyntax(String),
    /// Contains invalid number
    #[error("command contains invalid number")]
    ParseNumberError(#[from] ParseIntError),
    /// Roll dice too much times
    #[error("dice roll times limit exceeded")]
    DiceRollTimesLimitExceeded,
    /// Dice have too many sides
    #[error("dice sides count limit exceeded")]
    DiceSidesCountLimitExceeded,
    /// Too many items in expr
    #[error("items count limit exceeded")]
    ItemCountLimitExceeded,
    /// Number item too large
    #[error("number item too large")]
    NumberItemTooLarge,
}

impl<R: pest::RuleType> From<pest::error::Error<R>> for GurgleError {
    fn from(err: pest::error::Error<R>) -> Self {
        Self::InvalidSyntax(format!("{}", err))
    }
}
