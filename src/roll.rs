//! rolling result

use std::sync::atomic::{AtomicPtr, Ordering};

use crate::{
    ast::{Operator, PostProcessor},
    checker::Checker,
    tree::{BinaryTree, BinaryTreeNode},
};

fn cache_it<T, F>(cache: &AtomicPtr<T>, f: F) -> T
where
    T: Copy,
    F: FnOnce() -> T,
{
    let x = cache.load(Ordering::SeqCst);
    if x.is_null() {
        let value = f();
        let p = Box::into_raw(Box::new(value));
        match cache.compare_exchange(
            std::ptr::null::<T>() as *mut T,
            p,
            Ordering::SeqCst,
            Ordering::SeqCst,
        ) {
            Ok(_) => {}
            // Safety:
            // 1. cache value is stored only in this method, by `Box::into_raw`, so the ptr is valid
            Err(last) => drop(unsafe { Box::from_raw(last) }),
        }
        value
    } else {
        // Safety:
        // 1. cache value is stored only in this method, by `Box::into_raw`, so ptr is valid
        // 2. if cache has a value, it will not be de-allocated until struct drop, so it's ok to dereference it
        unsafe { *x }
    }
}

/// Roll result of a gurgle [`Dice`]
///
/// [`Dice`]: ../struct.Dice.html
#[derive(Debug)]
pub struct DiceRoll {
    points: Vec<u64>,
    pp: PostProcessor,
    cache: AtomicPtr<u64>,
}

impl DiceRoll {
    pub(crate) fn new(points: Vec<u64>, pp: PostProcessor) -> Self {
        Self {
            points,
            pp,
            cache: AtomicPtr::default(),
        }
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

    #[allow(clippy::missing_panics_doc)] // because this can't panic
    #[must_use]
    fn real_result(&self) -> u64 {
        match self.pp {
            PostProcessor::Sum => self.points.iter().sum(),
            PostProcessor::Avg => self.points.iter().sum::<u64>() / self.points.len() as u64,
            PostProcessor::Max => *self.points.iter().max().unwrap(),
            PostProcessor::Min => *self.points.iter().min().unwrap(),
        }
    }

    /// get the result, after post processor
    pub fn result(&self) -> u64 {
        cache_it(&self.cache, || self.real_result())
    }
}

/// Roll result of a gurgle expr tree [`Item`]
///
/// [`Item`]: ../ast/enum.Item.html
#[derive(Debug)]
pub enum RollItem {
    /// A dice item roll result
    Dice(DiceRoll),
    /// A const number item
    Number(i64),
}

impl RollItem {
    /// Get roll item result value
    #[must_use]
    pub fn result(&self) -> i64 {
        match self {
            #[allow(clippy::cast_possible_wrap)] // because out number can't be so big
            Self::Dice(dice) => dice.result() as i64,
            Self::Number(x) => *x,
        }
    }
}

/// Rolling result tree
pub type RollTree = BinaryTree<RollItem, Operator, AtomicPtr<i64>>;

impl RollTree {
    fn real_result(&self) -> i64 {
        match self.mid {
            Operator::Add => self.left.result() + self.right.result(),
            Operator::Minus => self.left.result() - self.result(),
        }
    }

    /// Get roll result
    pub fn result(&self) -> i64 {
        cache_it(&self.extra, || self.real_result())
    }
}

/// Rolling result tree item
pub type RollTreeNode = BinaryTreeNode<RollItem, Operator, AtomicPtr<i64>>;

impl RollTreeNode {
    pub fn result(&self) -> i64 {
        match self {
            Self::Leaf(leaf) => leaf.result(),
            Self::Tree(tree) => tree.result(),
        }
    }
}

/// Gurgle roll result
#[derive(Debug)]
pub struct GurgleRoll<'g> {
    result: RollTreeNode,
    checker: Option<&'g Checker>,
    cache: AtomicPtr<i64>,
}

impl<'g> GurgleRoll<'g> {
    pub(crate) fn new(result: RollTreeNode, checker: Option<&'g Checker>) -> Self {
        Self {
            result,
            checker,
            cache: AtomicPtr::default(),
        }
    }

    /// Get result expr
    #[must_use]
    pub const fn expr(&self) -> &RollTreeNode {
        &self.result
    }

    /// Get result
    #[must_use]
    pub fn result(&self) -> i64 {
        cache_it(&self.cache, || self.result.result())
    }

    /// Check if this result is a success(passed) roll
    pub fn success(&self) -> Option<bool> {
        self.checker.map(|c| c.check(self.result()))
    }
}
