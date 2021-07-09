//! Abstract syntax tree of gurgle expr

use std::str::FromStr;

use pest::iterators::Pair;

use crate::{
    config::Config,
    error::{GurgleError, ParseEnumError},
    parser::Rule,
};

/// Post process action after a round of dice roll
///
/// ## Example
///
/// - `3d6` default to sum of roll a 6 sides dice 3 times
/// - `3d6max` means get the max value of those 3 result
/// - `3d6min` means get the min value of those 3 result
/// - `3d6avg` means get the avg value of those 3 result
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PostProcessor {
    /// get sum of all roll, default action
    Sum,
    /// get avg value of all roll
    Avg,
    /// get max value of all roll
    Max,
    /// get min value of all roll
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

/// A dice roll action
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Dice {
    /// roll dice how many times
    pub times: u64,
    /// side count of this dice
    pub sided: u64,
    /// post process action after all roll, see [`PostProcessor`]
    ///
    /// [`PostProcessor`]: enum.PostProcessor.html
    pub pp: PostProcessor,
}

impl Dice {
    #[allow(clippy::cast_sign_loss)] // because times and sided can't be negative
    fn from_pair(pair: Pair<'_, Rule>, config: &Config) -> Result<Self, GurgleError> {
        assert_eq!(pair.as_rule(), Rule::dice);

        let mut pairs = pair.into_inner();
        let times = pairs.next().unwrap().as_str().parse::<i64>()?;
        let sided = pairs.next().unwrap().as_str().parse::<i64>()?;
        if times <= 0 || sided <= 0 {
            return Err(GurgleError::DiceRollOrSidedNegative);
        }
        if times as u64 > config.max_roll_times {
            return Err(GurgleError::DiceRollTimesLimitExceeded);
        }
        if sided as u64 > config.max_dice_sides {
            return Err(GurgleError::DiceSidedCountLimitExceeded);
        }
        let pp = pairs
            .next()
            .map_or(PostProcessor::Sum, |s| s.as_str().parse().unwrap());

        Ok(Self {
            times: times as u64,
            sided: sided as u64,
            pp,
        })
    }
}

/// Item in gurgle expr, can be a number or a dice roll action
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Item {
    /// A normal number
    Number(i64),
    /// A dice
    Dice(Dice),
}

impl Item {
    fn from_pair(pair: Pair<'_, Rule>, config: &Config) -> Result<Self, GurgleError> {
        assert_eq!(pair.as_rule(), Rule::item);

        let expr = pair.into_inner().next().unwrap();

        let result = match expr.as_rule() {
            #[allow(clippy::cast_sign_loss)] // because x is >= 0 after check
            Rule::number => {
                let x = expr.as_str().parse::<i64>()?;
                if x.abs() as u64 > config.max_number_item_value {
                    return Err(GurgleError::NumberItemOutOfRange);
                }
                Self::Number(x)
            }
            Rule::dice => Self::Dice(Dice::from_pair(expr, config)?),
            _ => unreachable!(),
        };

        Ok(result)
    }
}

/// Operator in gurgle expr
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Operator {
    /// add left tree result and right tree result
    Add,
    /// minus left tree result with right tree result
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

        Ok(op)
    }
}

/// Gurgle expr tree
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Tree {
    /// Left tree
    pub left: Box<TreeNode>,
    /// operator
    pub op: Operator,
    /// right tree
    pub right: Box<TreeNode>,
}

/// Node in gurgle expr tree
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TreeNode {
    /// A single item
    Item(Item),
    /// A sub expr tree
    SubTree(Tree),
}

impl TreeNode {
    pub(crate) fn from_pair(pair: Pair<'_, Rule>, config: &Config) -> Result<Self, GurgleError> {
        let mut expr: Option<Self> = None;
        let mut op = None;
        let mut times_sum = 0;
        let mut item_count = 0;

        for pair in pair.into_inner() {
            match pair.as_rule() {
                Rule::item => {
                    let item = Item::from_pair(pair, config)?;
                    item_count += 1;
                    if item_count > config.max_item_count {
                        return Err(GurgleError::ItemCountLimitExceeded);
                    }
                    if let Item::Dice(Dice { times, .. }) = item {
                        times_sum += times;
                        if times_sum > config.max_roll_times {
                            return Err(GurgleError::DiceRollTimesLimitExceeded);
                        }
                    }
                    if expr.is_none() {
                        expr.replace(Self::Item(item));
                    } else {
                        let e = expr.take().unwrap();
                        expr.replace(Self::SubTree(Tree {
                            left: Box::new(e),
                            op: op.take().unwrap(),
                            right: Box::new(Self::Item(item)),
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
