//! Roll dice using TRPG-like syntax.

// ===== lint config =====

#![deny(clippy::all, clippy::pedantic, clippy::nursery)]
#![deny(missing_debug_implementations, rust_2018_idioms)]
#![deny(missing_docs)]
#![deny(warnings)]
#![allow(
    clippy::module_name_repetitions,
    clippy::cast_possible_truncation,
    clippy::non_ascii_literal
)]

// ===== mods =====

pub mod ast;
pub mod checker;
mod config;
pub mod error;
mod parser;
pub mod roll;
mod tree;

// ===== uses =====

use pest::Parser;

use crate::{
    ast::AstTreeNode,
    checker::Checker,
    error::GurgleError,
    parser::{GurgleParser, Rule},
    roll::GurgleRoll,
};

// ===== pub uses =====

pub use {ast::Dice, config::Config};

// ===== implement =====

/// Rolling result
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RollResult<'g> {
    result: i64,
    checker: Option<&'g Checker>,
}

impl<'g> RollResult<'g> {
    /// Get result
    #[must_use]
    pub const fn result(&self) -> i64 {
        self.result
    }

    /// Is a success rolling result
    #[must_use]
    pub fn success(&self) -> Option<bool> {
        self.checker.map(|c| c.check(self.result))
    }
}

/// Parsed struct of a gurgle rolling command
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Gurgle {
    expr: AstTreeNode,
    checker: Option<Checker>,
}

impl Gurgle {
    /// Compile `s` into a the inner structure for executing, with a custom limits configuration.
    ///
    /// ## Errors
    ///
    /// When parse failed(not valid gurgle syntax) or exceeded the limit defined in `config`.
    ///
    /// ## Panics
    ///
    /// Only when internal logic error, please report issue if happened.
    pub fn compile_with_config(s: &str, config: &Config) -> Result<Self, GurgleError> {
        let pairs = GurgleParser::parse(Rule::gurgle, s)?;

        let mut expr = None;
        let mut checker = None;

        for pair in pairs {
            match pair.as_rule() {
                Rule::expr => {
                    expr.replace(AstTreeNode::from_pair(pair, config)?);
                }
                Rule::checker => {
                    checker.replace(Checker::from_pair(pair, config)?);
                }
                Rule::EOI => {}
                _ => unreachable!(),
            }
        }

        Ok(Self {
            expr: expr.unwrap(),
            checker,
        })
    }

    /// Compile a string to gurgle expr, using [default config].
    ///
    /// ## Errors
    ///
    /// See [`compile_with_config`].
    ///
    /// [default config]: struct.config.html#method.default
    /// [`compile_with_config`]: #method.compile_with_config
    pub fn compile(s: &str) -> Result<Self, GurgleError> {
        Self::compile_with_config(s, &config::DEFAULT_CONFIG)
    }

    /// Get the gurgle expr ast tree root node for walk through
    #[must_use]
    pub const fn expr(&self) -> &AstTreeNode {
        &self.expr
    }

    /// Get the gurgle expr ast tree root node for walk through
    #[must_use]
    pub const fn checker(&self) -> Option<&Checker> {
        self.checker.as_ref()
    }

    /// Rolling the complied dice command and get result
    #[must_use]
    pub fn roll(&self) -> GurgleRoll<'_> {
        GurgleRoll::new(self.expr.roll(), self.checker())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_correct() {
        assert!(Gurgle::compile("1d6+1").is_ok());
        assert!(Gurgle::compile("3d6+2d10+1").is_ok());
        assert!(Gurgle::compile("3d6max+2d10min+1").is_ok());
        assert!(Gurgle::compile("3d6max+2d10min+1>=10").is_ok());
        assert!(Gurgle::compile("3d6max+2d10min+1>=-10").is_ok());
        assert!(Gurgle::compile("100d1000+-1").is_ok());
    }

    #[test]
    fn test_parser_invalid() {
        assert!(std::matches!(
            Gurgle::compile("+").unwrap_err(),
            GurgleError::InvalidSyntax(_)
        ));
        assert!(std::matches!(
            Gurgle::compile("good").unwrap_err(),
            GurgleError::InvalidSyntax(_)
        ));
        assert!(std::matches!(
            Gurgle::compile("1d6x1").unwrap_err(),
            GurgleError::InvalidSyntax(_)
        ));
        assert!(std::matches!(
            Gurgle::compile("3d6+2p10+1").unwrap_err(),
            GurgleError::InvalidSyntax(_)
        ));
        assert!(std::matches!(
            Gurgle::compile("3d6max+2d10min+1avg").unwrap_err(),
            GurgleError::InvalidSyntax(_)
        ));
        assert!(std::matches!(
            Gurgle::compile("3d6+100000000000000000000000000").unwrap_err(),
            GurgleError::ParseNumberError(_),
        ));
    }

    #[test]
    fn test_compile_error() {
        assert_eq!(
            Gurgle::compile("10d-10").unwrap_err(),
            GurgleError::DiceRollOrSidedNegative,
        );
        assert_eq!(
            Gurgle::compile("-10d10").unwrap_err(),
            GurgleError::DiceRollOrSidedNegative,
        );
        assert_eq!(
            Gurgle::compile(
                "3d6+3d6+3d6+3d6+3d6+3d6+3d6+3d6+3d6+3d6+3d6+3d6+3d6+3d6+3d6+3d6+3d6+3d6+3d6+3d6+1"
            )
            .unwrap_err(),
            GurgleError::ItemCountLimitExceeded,
        );
        assert_eq!(
            Gurgle::compile("10d1001").unwrap_err(),
            GurgleError::DiceSidedCountLimitExceeded,
        );
        assert_eq!(
            Gurgle::compile("1001d10").unwrap_err(),
            GurgleError::DiceRollTimesLimitExceeded,
        );
        assert_eq!(
            Gurgle::compile("1000d10+1d10").unwrap_err(),
            GurgleError::DiceRollTimesLimitExceeded,
        );
        assert_eq!(
            Gurgle::compile("65537").unwrap_err(),
            GurgleError::NumberItemOutOfRange,
        );
        assert_eq!(
            Gurgle::compile("-65537").unwrap_err(),
            GurgleError::NumberItemOutOfRange,
        );
    }

    #[test]
    fn test_roll() {
        let attack_dices = Gurgle::compile("3d6+2d2+2>12").unwrap();
        let attack = attack_dices.roll();
        println!("attack expr: {:?}", attack.expr());
        println!("attack = {}", attack.result());
        println!("success = {}", attack.success().unwrap());
    }
}
