use std::{cmp::Ordering, fmt::format, num::ParseIntError, str::FromStr};

#[macro_use]
extern crate pest_derive;
use pest::{iterators::Pair, Parser, RuleType};

use thiserror::Error;

mod error;

pub use error::ParseEnumError;

#[derive(Parser)]
#[grammar = "gurgle.pest"] // relative to src
struct GurgleParser {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum PostProcessor {
    Sum,
    Avg,
    Max,
    Min,
}

impl FromStr for PostProcessor {
    type Err = ParseEnumError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let res = match s.to_ascii_lowercase().as_str() {
            "sum" => Self::Sum,
            "avg" => Self::Avg,
            "max" => Self::Max,
            "min" => Self::Min,
            _ => return Err(ParseEnumError),
        };

        Ok(res)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Dice {
    times: u64,
    faces: u64,
    pp: PostProcessor,
}

impl Dice {
    fn from_pair(pair: Pair<Rule>) -> Result<Self, GurgleError> {
        assert_eq!(pair.as_rule(), Rule::dice);

        let mut pairs = pair.into_inner();
        let times = pairs.next().unwrap().as_str().parse()?;
        let faces = pairs.next().unwrap().as_str().parse()?;
        let pp = pairs
            .next()
            .map_or(PostProcessor::Sum, |s| s.as_str().parse().unwrap());

        Ok(Self { times, faces, pp })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Item {
    Number(u64),
    Dice(Dice),
}

impl Item {
    fn from_pair(pair: Pair<Rule>) -> Result<Self, GurgleError> {
        assert_eq!(pair.as_rule(), Rule::item);

        let expr = pair.into_inner().next().unwrap();

        let result = match expr.as_rule() {
            Rule::number => Self::Number(expr.as_str().parse::<u64>()?),
            Rule::dice => Self::Dice(Dice::from_pair(expr)?),
            _ => unreachable!(),
        };

        Ok(result)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Operator {
    Add,
    Minus,
}

impl FromStr for Operator {
    type Err = ParseEnumError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let op = match s {
            "+" => Self::Add,
            "-" => Self::Minus,
            _ => return Err(ParseEnumError),
        };

        return Ok(op);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Expr {
    left: Box<ExprNode>,
    op: Operator,
    right: Box<ExprNode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum ExprNode {
    Item(Item),
    Expr(Expr),
}

impl ExprNode {
    fn from_pair(pair: Pair<Rule>) -> Result<Self, GurgleError> {
        let mut expr: Option<ExprNode> = None;
        let mut op = None;

        for pair in pair.into_inner() {
            match pair.as_rule() {
                Rule::item => {
                    let item = Item::from_pair(pair)?;
                    if expr.is_none() {
                        expr.replace(ExprNode::Item(item));
                    } else {
                        let e = expr.take().unwrap();
                        expr.replace(ExprNode::Expr(Expr {
                            left: Box::new(e),
                            op: op.take().unwrap(),
                            right: Box::new(ExprNode::Item(item)),
                        }));
                    }
                }
                Rule::operator => {
                    op.replace(Operator::from_str(pair.as_str()).unwrap());
                }
                _ => {}
            }
        }

        Ok(expr.unwrap())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Compare {
    Gte,
    Gt,
    Lte,
    Lt,
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

        return Ok(cmp);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Checker {
    compare: Compare,
    target: u64,
}

impl Checker {
    fn from_pair(pair: Pair<Rule>) -> Result<Self, GurgleError> {
        assert_eq!(pair.as_rule(), Rule::checker);

        let mut pairs = pair.into_inner();
        let compare = pairs.next().unwrap().as_str().parse().unwrap();
        let target = pairs.next().unwrap().as_str().parse()?;

        Ok(Self { compare, target })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Gurgle {
    expr: ExprNode,
    checker: Option<Checker>,
}

#[derive(Debug, Error)]
enum GurgleError {
    #[error("invalid gurgle command: {0}")]
    ParseError(String),
    #[error("command contains invalid number")]
    ParseNumberError(#[from] ParseIntError),
    #[error("dice roll times reach limit")]
    ReachMaxRollTimes,
    #[error("dice count reach limit")]
    ReachMaxDiceCount,
}

impl<R: RuleType> From<pest::error::Error<R>> for GurgleError {
    fn from(err: pest::error::Error<R>) -> Self {
        Self::ParseError(format!("{}", err))
    }
}

impl Gurgle {
    pub fn new(s: &str) -> Result<Self, GurgleError> {
        let mut pairs = GurgleParser::parse(Rule::gurgle, s)?;

        let mut expr = None;
        let mut checker = None;

        for pair in pairs.next().unwrap().into_inner() {
            match pair.as_rule() {
                Rule::expr => {
                    expr.replace(ExprNode::from_pair(pair)?);
                }
                Rule::checker => {
                    checker.replace(Checker::from_pair(pair)?);
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use pest::Parser;

    #[test]
    fn test_parser_correct() {
        assert!(GurgleParser::parse(Rule::gurgle, "1d6+1").is_ok());
        assert!(GurgleParser::parse(Rule::gurgle, "3d6+2d10+1").is_ok());
        assert!(GurgleParser::parse(Rule::gurgle, "3d6max+2d10min+1").is_ok());

        let gurgle = Gurgle::new("3d6max+2d10min+1").unwrap();
        println!("{:?}", gurgle)
    }

    #[test]
    fn test_parser_invalid() {
        assert!(GurgleParser::parse(Rule::gurgle, "+").is_err());
        assert!(GurgleParser::parse(Rule::gurgle, "good").is_err());
        assert!(GurgleParser::parse(Rule::gurgle, "1d6x1").is_err());
        assert!(GurgleParser::parse(Rule::gurgle, "3d6+2p10+1").is_err());
        assert!(GurgleParser::parse(Rule::gurgle, "3d6max+2d10min+1avg").is_err())
    }
}
