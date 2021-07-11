/// Common binary tree structure
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BinaryTree<T, Mid = (), Extra = ()> {
    /// Left tree
    pub left: Box<BinaryTreeNode<T, Mid, Extra>>,
    /// middle value
    pub mid: Mid,
    /// right tree
    pub right: Box<BinaryTreeNode<T, Mid, Extra>>,

    pub(crate) extra: Extra,
}

impl<T, Mid, Extra> BinaryTree<T, Mid, Extra>
where
    Extra: Default,
{
    /// Create a new tree from left/right sub tree and middle(new root) value
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

/// Node in the binary tree
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BinaryTreeNode<T, Mid = (), Extra = ()> {
    /// A leaf node
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

    /// Try treat this node as a leaf node and get leaf value
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
