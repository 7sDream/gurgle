//! rolling result

use crate::{
    ast::{Operator, PostProcessor},
    tree::{BinaryTree, BinaryTreeNode},
};

/// Result of a dice roll
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DiceRoll {
    points: Vec<u64>,
    pp: PostProcessor,
}

impl DiceRoll {
    pub(crate) const fn new(points: Vec<u64>, pp: PostProcessor) -> Self {
        Self { points, pp }
    }

    /// get post processor
    #[must_use]
    pub const fn post_processor(&self) -> PostProcessor {
        self.pp
    }

    /// get result points
    #[must_use]
    pub fn points(&self) -> &[u64] {
        &self.points
    }

    /// get result points count
    #[allow(clippy::len_without_is_empty)] // because it can't be empty
    #[must_use]
    pub fn len(&self) -> usize {
        self.points.len()
    }

    /// get the result, after post processor
    #[allow(clippy::missing_panics_doc)] // because this can't panic
    #[must_use]
    pub fn result(&self) -> u64 {
        match self.pp {
            PostProcessor::Sum => self.points.iter().sum(),
            PostProcessor::Avg => self.points.iter().sum::<u64>() / self.points.len() as u64,
            PostProcessor::Max => *self.points.iter().max().unwrap(),
            PostProcessor::Min => *self.points.iter().min().unwrap(),
        }
    }
}

/// Roll result of a gurgle expr tree [`Item`]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RollItem {
    /// A dice item roll result
    Dice(DiceRoll),
    /// A const number item
    Number(i64),
}

/// Rolling result tree
pub type RollTree = BinaryTree<RollItem, Operator>;
/// Rolling result tree item
pub type RollTreeNode = BinaryTreeNode<RollItem, Operator>;
