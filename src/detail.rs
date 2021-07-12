//! show roll result in detail

use std::fmt::{Display, Formatter, Write};

use crate::{
    checker::{Checker, Compare},
    expr::{Operator, PostProcessor},
    roll::{DiceRoll, GurgleRoll, ItemRoll, RollTree, RollTreeNode},
};

impl Display for Checker {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self.compare {
            Compare::Gte => ">=",
            Compare::Gt => ">",
            Compare::Lte => "<=",
            Compare::Lt => "<",
            Compare::Eq => "=",
        })?;
        f.write_fmt(format_args!("{}", self.target))
    }
}

impl Display for DiceRoll {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let (prefix, mid, postfix) = match self.post_processor() {
            PostProcessor::Sum => ("", "+", ""),
            PostProcessor::Avg => ("Avg[", ",", "]"),
            PostProcessor::Max => ("Max[", ",", "]"),
            PostProcessor::Min => ("Min[", ",", "]"),
        };

        f.write_char('(')?;
        f.write_str(prefix)?;
        let last = self.len() - 1;
        for (i, value) in self.points().iter().enumerate() {
            f.write_fmt(format_args!("{}", value))?;
            if i != last {
                f.write_str(mid)?;
            }
        }
        f.write_str(postfix)?;
        if self.post_processor() != PostProcessor::Avg {
            f.write_fmt(format_args!("={}", self.value()))?
        }
        f.write_char(')')
    }
}

impl Display for ItemRoll {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number(x) => f.write_fmt(format_args!("{}", x)),
            Self::Dice(dice) => f.write_fmt(format_args!("{}", dice)),
            Self::Parentheses(e) => f.write_fmt(format_args!("({})", e.as_ref())),
        }
    }
}

impl Display for RollTree {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let op = match self.mid {
            Operator::Add => "+",
            Operator::Minus => "-",
            Operator::Multiply => "*",
        };
        f.write_fmt(format_args!("{} {} {}", self.left, op, self.right))
    }
}

impl Display for RollTreeNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Leaf(leaf) => f.write_fmt(format_args!("{}", leaf)),
            Self::Tree(tree) => f.write_fmt(format_args!("{}", tree)),
        }
    }
}

impl Display for GurgleRoll<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.expr()))?;

        if !std::matches!(self.expr(), RollTreeNode::Leaf(ItemRoll::Number(_))) {
            f.write_fmt(format_args!(" = {}", self.value()))?;
        }

        if let Some(c) = self.checker() {
            f.write_fmt(format_args!(", target is {}", c))?;
            if self.success().unwrap() {
                f.write_str(", success")?;
            } else {
                f.write_str(", failed")?;
            }
        }
        Ok(())
    }
}
