//! rolling result

use std::sync::atomic::{AtomicPtr, Ordering};

use crate::{
    checker::Checker,
    expr::{Operator, PostProcessor},
    tree::{BinaryTree, BinaryTreeNode},
};

// Safety:
// 1. You should only change `cache` value by calling this method
unsafe fn cache_it<T, F>(cache: &AtomicPtr<T>, f: F) -> T
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
            // a success exchange, return value is a null ptr, so no need to deallocate
            Ok(_) => {}
            // Safety:
            // Because of function safety requirement,
            // cache value is stored only in this method, by `Box::into_raw`, so the ptr is valid
            Err(p) => drop(Box::from_raw(p)),
        }
        value
    } else {
        // Safety:
        // Because of function safety requirement,
        // cache value is stored only in this method, by `Box::into_raw`, so ptr is valid.
        // And if cache has a value, it will not change again, so gotten value is alive(until cache itself be dropped),
        // so it's ok to dereference it.
        *x
    }
}

/// Rolling result of a gurgle [`Dice`]
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

    /// Get post processor
    #[must_use]
    pub const fn post_processor(&self) -> PostProcessor {
        self.pp
    }

    /// Get rolling dice output points
    #[must_use]
    pub fn points(&self) -> &[u64] {
        &self.points
    }

    /// Get points count(rolling dice times)
    #[allow(clippy::len_without_is_empty)] // because it can't be empty
    #[must_use]
    pub fn len(&self) -> usize {
        self.points.len()
    }

    #[allow(clippy::missing_panics_doc)] // because this can't panic
    #[must_use]
    fn calculate_value(&self) -> u64 {
        match self.pp {
            PostProcessor::Sum => self.points.iter().sum(),
            PostProcessor::Avg => self.points.iter().sum::<u64>() / self.points.len() as u64,
            PostProcessor::Max => *self.points.iter().max().unwrap(),
            PostProcessor::Min => *self.points.iter().min().unwrap(),
        }
    }

    /// Get the final rolling result value, with post processor executed
    pub fn value(&self) -> u64 {
        // Safety: `cache` only used in `cache_it` function
        unsafe { cache_it(&self.cache, || self.calculate_value()) }
    }
}

/// Rolling result of a gurgle expression tree [`Item`]
///
/// [`Item`]: ../ast/enum.Item.html
#[derive(Debug)]
pub enum ItemRoll {
    /// rolling result of a dice item
    Dice(DiceRoll),
    /// number item, rolling result is itself
    Number(i64),
    /// rolling result of another sub expr, which is commonly wrapped by parentheses
    Parentheses(Box<RollTreeNode>),
}

impl ItemRoll {
    /// Get rolling result value
    #[must_use]
    pub fn value(&self) -> i64 {
        match self {
            #[allow(clippy::cast_possible_wrap)] // because out number can't be so big
            Self::Dice(dice) => dice.value() as i64,
            Self::Number(x) => *x,
            Self::Parentheses(e) => e.value(),
        }
    }
}

/// Rolling result tree
pub type RollTree = BinaryTree<ItemRoll, Operator, AtomicPtr<i64>>;

impl RollTree {
    fn calculate_value(&self) -> i64 {
        match self.mid {
            Operator::Add => self.left.value() + self.right.value(),
            Operator::Minus => self.left.value() - self.right.value(),
            Operator::Multiply => self.left.value() * self.right.value(),
        }
    }

    /// Get rolling result value
    pub fn value(&self) -> i64 {
        // Safety: `cache` only used in `cache_it` function
        unsafe { cache_it(&self.extra, || self.calculate_value()) }
    }
}

/// Rolling result tree node, can be a leaf or a sub tree
pub type RollTreeNode = BinaryTreeNode<ItemRoll, Operator, AtomicPtr<i64>>;

impl RollTreeNode {
    /// Get rolling result value
    pub fn value(&self) -> i64 {
        match self {
            Self::Leaf(leaf) => leaf.value(),
            Self::Tree(tree) => tree.value(),
        }
    }
}

/// Rolling result of [`Gurgle`] command
///
/// [`Gurgle`]: ../struct.Gurgle.html
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

    /// Get rolling result expression
    #[must_use]
    pub const fn expr(&self) -> &RollTreeNode {
        &self.result
    }

    /// Get the checker
    pub const fn checker(&self) -> Option<&'g Checker> {
        self.checker
    }

    /// Get rolling result value
    #[must_use]
    pub fn value(&self) -> i64 {
        // Safety: cache only used in cache_it
        unsafe { cache_it(&self.cache, || self.result.value()) }
    }

    /// Check if this rolling result is success(passed)
    pub fn success(&self) -> Option<bool> {
        self.checker.map(|c| c.check(self.value()))
    }
}
