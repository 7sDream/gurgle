//! Rolling dice using TRPG-like syntax.
//!
//! ## Example
//!
//! ### Only need result value
//!
//! ```rust
//! let attack = "3d6+2d4+1";
//! println!("roll your attack({}), result: {}", attack, gurgle::roll(attack).unwrap());
//!
//! // output: roll your attack(3d6+2d4+1), result: 16
//! ```
//!
//! ### Need check if rolling result is success(pass)
//!
//! ```rust
//! use gurgle::Gurgle;
//!
//! let attack = "3d6+2d4+1>15";
//! let dice = Gurgle::compile(attack).unwrap();
//! let result = dice.roll();
//!
//! println!(
//!     "roll your attack({}), result: {}, {}",
//!     attack, result.value(),
//!     if result.success().unwrap() { "success" } else { "miss" },
//! );
//!
//! // output: roll your attack(3d6+2d4+1>15), result: 16, success
//! ```
//!
//! ### Need get rolling result of every dice
//!
//! ```rust
//! use gurgle::Gurgle;
//!
//! let attack = "3d6+2d4+1>15";
//! let dice = Gurgle::compile(attack).unwrap();
//! let result = dice.roll();
//!
//! println!("roll your attack({}), result: {}", attack, result);
//!
//! // output: roll your attack(3d6+2d4+1>15), result: (4+3+1) + (1+3) + 1 = 15, target is >15, failed
//! ```
//!
//! Notice: `Display` trait for rolling result is implemented only if
//! feature `detail`(which is enabled by default) is enabled.
//!
//! You can see source code `detail.rs` for how to can walk through result tree
//! and construct you own output message format.
//!
//! ## Command Syntax
//!
//! A Gurgle command is consists of two parts: dice expression([`AstTreeNode`]) and a optional [`Checker`].
//!
//! Dice expression is addition or minus of one or more item, item can be a const number or a dice rolling round.
//!
//! Dice rolling round can be write as `x`d`y`: `x` is rolling times, `y` is dice sided,
//! so it means rolling a `y` sided dice `x` times and sum the result points.
//!
//! In addition to summing, a dice rolling round can use `avg`, `max`, and `min` to get the final result of this round.
//!
//! Some example for easily understand:
//!
//! - ✅️ `3d6`
//! - ✅️ `3d6+1`
//! - ✅️ `3d6+2d4+2`
//! - ✅️ `3d6max+2d4max+1`
//! - ✅️ `3d6max + 2d4max + 1`，space between item and operator(`+`/`-`) is acceptable
//! - ❌️ `3d6 max`, space can't appear in inner of a item
//! - ❌️ `0d-10`, `x` and `y` can't be zero or negative value
//! - ✅️ `2d10-3d2-1`, minus ok
//! - ✅️ `2d10*3+4`, multiply ok
//! - ✅️ `(2d6+1)*4+1`, parentheses ok
//!
//! And you can add checker, it a compare with a value, that is, right side of a (in)equation:
//!
//! - `>=10`
//! - `>10`
//! - `<=10`
//! - `<10`
//! - `=10`
//!
//! A full example: `3d6+(2d4+1)*2+1 > 20`.
//!
//! space between expr and checker, between compare and value is optional.
//!
//! So it's the same as: `3d6+(2d4+1)*2+1>20`.
//!
//! [`AstTreeNode`]: expr/type.AstTreeNode.html
//! [`Checker`]: checker/struct.Checker.html

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

pub mod checker;
mod config;
#[cfg(feature = "detail")]
pub mod detail;
pub mod error;
pub mod expr;
mod parser;
pub mod roll;
mod tree;

// ===== uses =====

use config::Limit;
use pest::Parser;

use crate::{
    checker::Checker,
    error::CompileError,
    expr::AstTreeNode,
    parser::{GurgleCommandParser, Rule},
    roll::GurgleRoll,
};

// ===== pub uses =====

pub use {config::Config, expr::Dice};

// ===== implement =====

/// A Compiled gurgle command
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Gurgle {
    expr: AstTreeNode,
    checker: Option<Checker>,
}

impl Gurgle {
    /// Compile string `s` to a gurgle command, with a custom limits configuration.
    ///
    /// ## Errors
    ///
    /// When parse failed(invalid gurgle syntax, etc) or exceeded the limit defined in `config`.
    #[allow(clippy::missing_panics_doc)] // because unreachable branch is indeed unreachable
    pub fn compile_with_config(s: &str, config: &Config) -> Result<Self, CompileError> {
        let mut limit = Limit::new(config);
        let pairs = GurgleCommandParser::parse(Rule::command, s)?;

        let mut expr = None;
        let mut checker = None;

        for pair in pairs {
            match pair.as_rule() {
                Rule::expr => {
                    expr.replace(AstTreeNode::from_pair(pair, &mut limit)?);
                }
                Rule::checker => {
                    checker.replace(Checker::from_pair(pair, &limit)?);
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

    /// Compile string `s` to a gurgle command, using [default config].
    ///
    /// ## Errors
    ///
    /// See [`compile_with_config`].
    ///
    /// [default config]: struct.config.html#method.default
    /// [`compile_with_config`]: #method.compile_with_config
    pub fn compile(s: &str) -> Result<Self, CompileError> {
        Self::compile_with_config(s, &config::DEFAULT_CONFIG)
    }

    /// Get the gurgle expression ast tree root node for walk through
    #[must_use]
    pub const fn expr(&self) -> &AstTreeNode {
        &self.expr
    }

    /// Get the gurgle checker
    #[must_use]
    pub const fn checker(&self) -> Option<&Checker> {
        self.checker.as_ref()
    }

    /// Rolling the compiled command and get result
    #[must_use]
    pub fn roll(&self) -> GurgleRoll<'_> {
        GurgleRoll::new(self.expr.roll(), self.checker())
    }
}

/// Compile then execute a gurgle command immediately, get result value
///
/// This function only gives you dice result value, but not check result.
/// If you need success check, use [`Gurgle::roll`] instead.
///
/// ## Errors
///
/// If compile `s` as a gurgle command failed, see [`Gurgle::compile`].
///
/// [`Gurgle::roll`]: struct.Gurgle.html#method.roll
/// [`Gurgle::compile`]: struct.Gurgle.html#method.compile
pub fn roll(s: &str) -> Result<i64, CompileError> {
    Gurgle::compile(s).map(|x| x.roll().value())
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
        assert!(Gurgle::compile("100d1000*5").is_ok());
        assert!(Gurgle::compile("10d1000x1d10").is_ok());
        assert!(Gurgle::compile("(10d1000)+(1)").is_ok());
        assert!(Gurgle::compile("3d6 + (2d4 + 1) * 2 + 1>20").is_ok());
        assert!(Gurgle::compile("3d6+(2d4+1)*2+1 >20").is_ok());
        assert!(Gurgle::compile("3d6+(2d4+1)*2+1> 20").is_ok());
        assert!(Gurgle::compile("3d6+(2d4+1)*2+1 > 20").is_ok());
    }

    #[test]
    fn test_parser_invalid() {
        assert!(std::matches!(
            Gurgle::compile("+").unwrap_err(),
            CompileError::InvalidSyntax(_)
        ));
        assert!(std::matches!(
            Gurgle::compile("good").unwrap_err(),
            CompileError::InvalidSyntax(_)
        ));
        assert!(std::matches!(
            Gurgle::compile("3d6+2p10+1").unwrap_err(),
            CompileError::InvalidSyntax(_)
        ));
        assert!(std::matches!(
            Gurgle::compile("3d6max+2d10min+1avg").unwrap_err(),
            CompileError::InvalidSyntax(_)
        ));
        assert!(std::matches!(
            Gurgle::compile("3d6+(1").unwrap_err(),
            CompileError::InvalidSyntax(_),
        ));
        assert!(std::matches!(
            Gurgle::compile("3d6 max+2d10min+1avg").unwrap_err(),
            CompileError::InvalidSyntax(_)
        ));
        assert!(std::matches!(
            Gurgle::compile("3d6+100000000000000000000000000").unwrap_err(),
            CompileError::ParseNumberError(_),
        ));
    }

    #[test]
    fn test_compile_error() {
        assert_eq!(
            Gurgle::compile("10d-10").unwrap_err(),
            CompileError::DiceRollOrSidedNegative,
        );
        assert_eq!(
            Gurgle::compile("-10d10").unwrap_err(),
            CompileError::DiceRollOrSidedNegative,
        );
        assert_eq!(
            Gurgle::compile(
                "3d6+3d6+3d6+3d6+3d6+3d6+3d6+3d6+3d6+3d6+3d6+3d6+3d6+3d6+3d6+3d6+3d6+3d6+3d6+3d6+1"
            )
            .unwrap_err(),
            CompileError::ItemCountLimitExceeded,
        );
        assert_eq!(
            Gurgle::compile("10d1001").unwrap_err(),
            CompileError::DiceSidedCountLimitExceeded,
        );
        assert_eq!(
            Gurgle::compile("1001d10").unwrap_err(),
            CompileError::DiceRollTimesLimitExceeded,
        );
        assert_eq!(
            Gurgle::compile("1000d10+1d10").unwrap_err(),
            CompileError::DiceRollTimesLimitExceeded,
        );
        assert_eq!(
            Gurgle::compile("65537").unwrap_err(),
            CompileError::NumberItemOutOfRange,
        );
        assert_eq!(
            Gurgle::compile("-65537").unwrap_err(),
            CompileError::NumberItemOutOfRange,
        );
    }

    #[test]
    fn test_roll() {
        // detail::Language::set_global(detail::Language::ZhCN);
        // detail::Language::set_global_custom(detail::OutputSpans {
        //     comma: "| ".into(),
        //     target_is: "we want".into(),
        //     success: "passed".into(),
        //     failed: "over".into(),
        // });
        let attack = Gurgle::compile("3d6min+3d6avg+3d6max+3d6+(2d4+1)*2+1>15").unwrap();
        let result = attack.roll();

        #[cfg(feature = "detail")]
        println!("attack rolling result is: {}", result);

        println!("attack = {}", result.value());
        assert!(result.value() >= 13);
        assert_eq!(result.success().unwrap(), result.value() > 15);
    }
}
