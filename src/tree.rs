/// Gurgle expr tree
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BinaryTree<T, Mid = (), Extra = ()> {
    /// Left tree
    pub left: Box<BinaryTreeNode<T, Mid, Extra>>,
    /// operator
    pub mid: Mid,
    /// right tree
    pub right: Box<BinaryTreeNode<T, Mid, Extra>>,

    pub(crate) extra: Extra,
}

impl<T, Mid, Extra> BinaryTree<T, Mid, Extra>
where
    Extra: Default,
{
    pub fn new(
        left: BinaryTreeNode<T, Mid, Extra>, right: BinaryTreeNode<T, Mid, Extra>, mid: Mid,
    ) -> Self {
        Self {
            left: Box::new(left),
            mid,
            right: Box::new(right),
            extra: Extra::default(),
        }
    }
}

/// Node in gurgle expr tree
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BinaryTreeNode<T, Mid = (), Extra = ()> {
    /// A single leaf item
    Leaf(T),
    /// A (sub) tree
    Tree(BinaryTree<T, Mid, Extra>),
}

impl<T, Mid, Extra> BinaryTreeNode<T, Mid, Extra> {
    /// Check if this node is a leaf node
    #[must_use]
    pub const fn is_leaf(&self) -> bool {
        std::matches!(self, Self::Leaf(_))
    }

    /// Check if this node is a tree
    #[must_use]
    pub const fn is_tree(&self) -> bool {
        std::matches!(self, Self::Tree(_))
    }

    /// Try treat this node as a leaf node and leaf value
    #[must_use]
    pub const fn as_leaf(&self) -> Option<&T> {
        match self {
            Self::Leaf(t) => Some(t),
            Self::Tree(_) => None,
        }
    }

    /// Try treat this node as a sub tree
    #[must_use]
    pub const fn as_tree(&self) -> Option<&BinaryTree<T, Mid, Extra>> {
        match self {
            Self::Leaf(_) => None,
            Self::Tree(t) => Some(t),
        }
    }
}
