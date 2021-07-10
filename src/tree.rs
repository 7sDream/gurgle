/// Gurgle expr tree
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BinaryTree<T, Mid = ()> {
    /// Left tree
    pub left: Box<BinaryTreeNode<T, Mid>>,
    /// operator
    pub mid: Mid,
    /// right tree
    pub right: Box<BinaryTreeNode<T, Mid>>,
}

impl<T, Mid> BinaryTree<T, Mid> {
    pub fn new(left: BinaryTreeNode<T, Mid>, right: BinaryTreeNode<T, Mid>, mid: Mid) -> Self {
        Self {
            left: Box::new(left),
            mid,
            right: Box::new(right),
        }
    }
}

/// Node in gurgle expr tree
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BinaryTreeNode<T, Mid = ()> {
    /// A single item
    Leaf(T),
    /// A sub expr tree
    SubTree(BinaryTree<T, Mid>),
}
