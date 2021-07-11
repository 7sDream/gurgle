//! show roll result in detail

use std::fmt::{Display, Formatter, Write};

use crate::{
    checker::Compare,
    expr::{Operator, PostProcessor},
    roll::{DiceRoll, GurgleRoll, ItemRoll, RollTree, RollTreeNode},
};

impl Display for DiceRoll {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let (prefix, mid) = match self.post_processor() {
            PostProcessor::Sum => ("", "+"),
            PostProcessor::Avg => ("Avg", ", "),
            PostProcessor::Max => ("Max", ", "),
            PostProcessor::Min => ("Min", ", "),
        };
        f.write_str(prefix)?;
        f.write_char('(')?;
        let last = self.len() - 1;
        for (i, value) in self.points().iter().enumerate() {
            f.write_fmt(format_args!("{}", value))?;
            if i != last {
                f.write_str(mid)?;
            }
        }
        f.write_char(')')
    }
}

impl Display for ItemRoll {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number(x) => f.write_fmt(format_args!("{}", x)),
            Self::Dice(dice) => f.write_fmt(format_args!("{}", dice)),
        }
    }
}

impl Display for RollTree {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let op = match self.mid {
            Operator::Add => "+",
            Operator::Minus => "-",
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
            f.write_str(" = ")?;
            f.write_fmt(format_args!("{}", self.value()))?;
        }

        if let Some(c) = self.checker() {
            f.write_str(", target is ")?;
            f.write_str(match c.compare {
                Compare::Gte => ">=",
                Compare::Gt => ">",
                Compare::Lte => "<=",
                Compare::Lt => "<",
                Compare::Eq => "=",
            })?;
            f.write_fmt(format_args!("{}", c.target))?;
            if self.success().unwrap() {
                f.write_str(", success")?;
            } else {
                f.write_str(", failed")?;
            }
        }
        Ok(())
    }
}
