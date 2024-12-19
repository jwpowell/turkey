use std::rc::Rc;

pub struct Tree<T>(Rc<TreeNode<T>>);

impl<T> Clone for Tree<T> {
    fn clone(&self) -> Self {
        Tree(Rc::clone(&self.0))
    }
}

struct TreeNode<T> {
    data: T,
    children: Vec<Tree<T>>,
}

impl<T> Tree<T> {
    pub fn node(data: T, children: &[Tree<T>]) -> Self {
        let children = children.to_vec();

        Tree(Rc::new(TreeNode { data, children }))
    }

    pub fn leaf(data: T) -> Self {
        Self::node(data, &[])
    }

    pub fn has_children(&self) -> bool {
        !self.0.children.is_empty()
    }

    pub fn is_leaf(&self) -> bool {
        !self.has_children()
    }

    pub fn children(&self) -> &[Self] {
        &self.0.children
    }

    pub fn data(&self) -> &T {
        &self.0.data
    }

    pub fn visit<V: TreeVisitor<T>>(
        &self,
        visitor: &mut V,
        preoder: bool,
        inorder: bool,
        postorder: bool,
    ) {
        let mut stack = vec![(None, self)];

        while let Some((traversal, node)) = stack.pop() {
            if let Some(traversal) = traversal {
                visitor.visit(node, traversal);
                continue;
            }

            if postorder {
                stack.push((Some(TreeTraversal::PostOrder), node));
            }

            for (i, child) in node.children().iter().enumerate().rev() {
                stack.push((None, child));

                if i != 0 && inorder {
                    stack.push((Some(TreeTraversal::InOrder), node));
                }
            }

            if preoder {
                stack.push((Some(TreeTraversal::PreOrder), node));
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum TreeTraversal {
    PreOrder,
    InOrder,
    PostOrder,
}

pub trait TreeVisitor<T> {
    fn visit(&mut self, node: &Tree<T>, traversal: TreeTraversal);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct TestVisitor {
        visited: Vec<(i32, TreeTraversal)>,
    }

    impl TestVisitor {
        fn new() -> Self {
            TestVisitor {
                visited: Vec::new(),
            }
        }
    }

    impl TreeVisitor<i32> for TestVisitor {
        fn visit(&mut self, node: &Tree<i32>, traversal: TreeTraversal) {
            self.visited.push((*node.data(), traversal));
        }
    }

    #[test]
    fn test_tree_creation() {
        let leaf = Tree::leaf(1);
        assert_eq!(*leaf.data(), 1);
        assert!(leaf.is_leaf());

        let node = Tree::node(2, &[leaf.clone()]);
        assert_eq!(*node.data(), 2);
        assert!(node.has_children());
        assert_eq!(node.children().len(), 1);
        assert_eq!(*node.children()[0].data(), 1);
    }

    #[test]
    fn test_tree_clone() {
        let leaf = Tree::leaf(1);
        let node = Tree::node(2, &[leaf.clone()]);
        let cloned_node = node.clone();

        assert_eq!(*cloned_node.data(), 2);
        assert!(cloned_node.has_children());
        assert_eq!(cloned_node.children().len(), 1);
        assert_eq!(*cloned_node.children()[0].data(), 1);
    }

    #[test]
    fn test_tree_traversal() {
        let leaf1 = Tree::leaf(1);
        let leaf2 = Tree::leaf(2);
        let node = Tree::node(3, &[leaf1.clone(), leaf2.clone()]);

        let mut visitor = TestVisitor::new();
        node.visit(&mut visitor, true, true, true);

        assert_eq!(
            visitor.visited,
            vec![
                (3, TreeTraversal::PreOrder),
                (1, TreeTraversal::PreOrder),
                (1, TreeTraversal::PostOrder),
                (3, TreeTraversal::InOrder),
                (2, TreeTraversal::PreOrder),
                (2, TreeTraversal::PostOrder),
                (3, TreeTraversal::PostOrder),
            ]
        );
    }

    #[test]
    fn test_tree_inorder_traversal() {
        let leaf1 = Tree::leaf(1);
        let leaf2 = Tree::leaf(2);
        let node = Tree::node(3, &[leaf1.clone(), leaf2.clone()]);

        let mut visitor = TestVisitor::new();
        node.visit(&mut visitor, false, true, false);

        assert_eq!(visitor.visited, vec![(2, TreeTraversal::InOrder),]);
    }
}
