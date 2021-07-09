//! Check whether a roll result is a success

use std::str::FromStr;

use pest::iterators::Pair;

use crate::{
    config::Config,
    error::{GurgleError, ParseEnumError},
    parser::Rule,
};

/// Compare operator in [`Checker`]
///
/// [`Checker`]: struct.Checker.html
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Compare {
    /// Grater then or equal
    Gte,
    /// Grater then
    Gt,
    /// Less then or equal
    Lte,
    /// Less then
    Lt,
    /// Equal
    Eq,
}

impl FromStr for Compare {
    type Err = ParseEnumError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let cmp = match s {
            ">=" => Self::Gte,
            ">" => Self::Gt,
            "<=" => Self::Lte,
            "<" => Self::Lt,
            "=" | "==" => Self::Eq,
            _ => return Err(ParseEnumError),
        };

        Ok(cmp)
    }
}

/// Check if the result of the gurgle execution is passed
///
/// `Checker` compare gurgle execution result to [`target`].
/// It's a success(pass) if compare result is as same as [`compare`] field.
///
/// ## Example
///
/// Gurgle command `3d6 > 10`: `>` is the [`compare`] and `10` is the [`target`].
/// When sum of 3 dice roll result grater then 10, it's a success(pass).
///
/// [`compare`]: #structfield.compare
/// [`target`]: #structfield.target
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Checker {
    /// wanted compare result
    pub compare: Compare,
    /// target value
    pub target: i64,
}

impl Checker {
    pub(crate) fn from_pair(pair: Pair<'_, Rule>, config: &Config) -> Result<Self, GurgleError> {
        assert_eq!(pair.as_rule(), Rule::checker);

        let mut pairs = pair.into_inner();
        let compare = pairs.next().unwrap().as_str().parse().unwrap();
        let target = pairs.next().unwrap().as_str().parse::<i64>()?;

        if target.abs() as u64 > config.max_number_item_value {
            return Err(GurgleError::NumberItemOutOfRange);
        }

        Ok(Self { compare, target })
    }

    /// Check if a rolling result is a success.
    #[must_use]
    pub fn check(&self, result: i64) -> bool {
        match result.cmp(&self.target) {
            std::cmp::Ordering::Greater => {
                std::matches!(self.compare, Compare::Gte | Compare::Gt)
            }
            std::cmp::Ordering::Less => {
                std::matches!(self.compare, Compare::Lte | Compare::Lt)
            }
            std::cmp::Ordering::Equal => {
                std::matches!(self.compare, Compare::Gte | Compare::Lte | Compare::Eq)
            }
        }
    }
}
