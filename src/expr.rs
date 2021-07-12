//! gurgle expression

use std::str::FromStr;

use nanorand::Rng;
use once_cell::sync::Lazy;
use pest::{
    iterators::Pair,
    prec_climber::{Assoc, Operator as PCOperator, PrecClimber},
};

static CLIMBER: Lazy<PrecClimber<Rule>> = Lazy::new(|| {
    PrecClimber::new(vec![
        PCOperator::new(Rule::op_add, Assoc::Left) | PCOperator::new(Rule::op_sub, Assoc::Left),
        PCOperator::new(Rule::op_multiply, Assoc::Left),
    ])
});

use crate::{
    config::Limit,
    error::{CompileError, ParseEnumError},
    parser::Rule,
    roll::{DiceRoll, ItemRoll, RollTree, RollTreeNode},
    tree::{BinaryTree, BinaryTreeNode},
};

/// Post processing action after a round of dice roll
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
    /// get avg value of all roll, will round down(floor) if not divisible
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

/// Rule of a round of dice roll
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Dice {
    /// roll dice how many times
    pub times: u64,
    /// side count of this dice
    pub sided: u64,
    /// post processing action after all roll, see [`PostProcessor`]
    ///
    /// [`PostProcessor`]: enum.PostProcessor.html
    pub pp: PostProcessor,
}

impl Dice {
    /// Create a new `n` sided dice and roll it `m` times, with default post processor [`Sum`]
    ///
    /// [`Sum`]: enum.PostProcessor.html#variant.Sum
    #[must_use]
    pub const fn new(n: u64, m: u64) -> Self {
        Self::new_with_pp(n, m, PostProcessor::Sum)
    }

    /// Create a new `n` sided dice and roll it `m` times, with post processor `pp`
    #[must_use]
    pub const fn new_with_pp(n: u64, m: u64, pp: PostProcessor) -> Self {
        Self {
            times: n,
            sided: m,
            pp,
        }
    }

    #[allow(clippy::cast_sign_loss)] // because times and sided can't be negative after check_dice
    fn from_pair(pair: Pair<'_, Rule>, limit: &mut Limit<'_>) -> Result<Self, CompileError> {
        assert_eq!(pair.as_rule(), Rule::dice);

        limit.inc_item_count()?;

        let mut pairs = pair.into_inner();
        let times = pairs.next().unwrap().as_str().parse::<i64>()?;
        let sided = pairs.next().unwrap().as_str().parse::<i64>()?;

        limit.check_dice(times, sided)?;
        limit.inc_roll_times(times as u64)?;

        let pp = pairs
            .next()
            .map_or(PostProcessor::Sum, |s| s.as_str().parse().unwrap());

        Ok(Self {
            times: times as u64,
            sided: sided as u64,
            pp,
        })
    }

    /// Roll a round of dice and get a result
    #[must_use]
    pub fn roll(&self) -> DiceRoll {
        let points = (0..self.times)
            .map(|_| nanorand::tls_rng().generate_range(1..=self.sided))
            .collect();
        DiceRoll::new(points, self.pp)
    }
}

/// Item in gurgle expression, can be a number or a dice
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Item {
    /// A normal number
    Number(i64),
    /// A dice
    Dice(Dice),
    /// Another expr wrapped by parentheses
    Parentheses(Box<AstTreeNode>),
}

impl Item {
    fn from_pair(pair: Pair<'_, Rule>, limit: &mut Limit<'_>) -> Result<Self, CompileError> {
        assert_eq!(pair.as_rule(), Rule::item);

        let expr = pair.into_inner().next().unwrap();

        let result = match expr.as_rule() {
            Rule::number => {
                limit.inc_item_count()?;
                let x = expr.as_str().parse::<i64>()?;
                limit.check_number_item(x)?;
                Self::Number(x)
            }
            Rule::dice => Self::Dice(Dice::from_pair(expr, limit)?),
            Rule::parentheses => Self::Parentheses(Box::new(AstTreeNode::from_pair(
                expr.into_inner().next().unwrap(),
                limit,
            )?)),
            _ => unreachable!(),
        };

        Ok(result)
    }

    /// Get roll result
    #[must_use]
    pub fn roll(&self) -> ItemRoll {
        match self {
            Self::Dice(d) => ItemRoll::Dice(d.roll()),
            Self::Number(x) => ItemRoll::Number(*x),
            Self::Parentheses(e) => ItemRoll::Parentheses(Box::new(e.roll())),
        }
    }

    /// Check if this item is a number
    #[must_use]
    pub const fn is_number(&self) -> bool {
        std::matches!(self, Item::Number(_))
    }

    /// Check if this item is a dice
    #[must_use]
    pub const fn is_dice(&self) -> bool {
        std::matches!(self, Item::Dice(_))
    }

    /// Check if this item is a expr
    #[must_use]
    pub const fn is_expr(&self) -> bool {
        std::matches!(self, Item::Parentheses(_))
    }

    /// Try treat this item as a number
    #[must_use]
    pub const fn as_number(&self) -> Option<i64> {
        match self {
            Self::Number(x) => Some(*x),
            _ => None,
        }
    }

    /// Try treat this item as a dice
    #[must_use]
    pub const fn as_dice(&self) -> Option<&Dice> {
        match self {
            Self::Dice(dice) => Some(dice),
            _ => None,
        }
    }

    /// Try treat this item as a dice
    #[must_use]
    pub const fn as_expr(&self) -> Option<&AstTreeNode> {
        match self {
            Self::Parentheses(e) => Some(e),
            _ => None,
        }
    }
}

/// Operator in gurgle expr
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Operator {
    /// add left tree result and right tree result
    Add,
    /// subtract the right tree result from the left result
    Minus,
    /// multiply left tree result with right tree result
    Multiply,
}

impl FromStr for Operator {
    type Err = ParseEnumError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let op = match s {
            "+" => Self::Add,
            "-" => Self::Minus,
            "x" | "*" => Self::Multiply,
            _ => return Err(ParseEnumError),
        };

        Ok(op)
    }
}

/// Abstract syntax tree of gurgle expr
pub type AstTree = BinaryTree<Item, Operator>;

impl AstTree {
    pub fn roll(&self) -> RollTree {
        RollTree::new(self.left.roll(), self.right.roll(), self.mid)
    }
}

/// Abstract syntax tree node, can be a leaf or a sub tree
pub type AstTreeNode = BinaryTreeNode<Item, Operator>;

impl AstTreeNode {
    pub(crate) fn from_pair(
        pair: Pair<'_, Rule>, limit: &mut Limit<'_>,
    ) -> Result<Self, CompileError> {
        let pairs = pair.into_inner();

        CLIMBER.climb(
            pairs,
            |p| {
                let item = Item::from_pair(p, limit)?;
                Ok(Self::Leaf(item))
            },
            |left, op, right| {
                Ok(Self::Tree(AstTree::new(
                    left?,
                    right?,
                    Operator::from_str(op.as_str()).unwrap(),
                )))
            },
        )
    }

    pub fn roll(&self) -> RollTreeNode {
        match self {
            Self::Leaf(item) => RollTreeNode::Leaf(item.roll()),
            Self::Tree(tree) => RollTreeNode::Tree(tree.roll()),
        }
    }
}
