/// Abstract syntax tree (AST) module for representing and manipulating Lisp-like expressions.
///
/// This module provides the core data structures and traits for working with AST nodes,
/// including support for different types of values (nil, pairs, symbols, numbers, etc.)
/// and tree traversal operations.
use std::rc::Rc;

use crate::utils::tree::*;

use num::BigInt;

/// Represents an AST node.
#[derive(Debug, Clone)]
pub struct Ast(Tree<AstNode>);

#[derive(Debug, Clone, Copy, Default)]
pub struct SyntaxInfo {}

/// Represents the different types of nodes that can exist in the AST.
#[derive(Debug, Clone, PartialEq)]
enum AstNode {
    /// Represents a nil value
    Nil,
    /// Represents a pair of AST nodes (head, tail)
    Pair,
    /// Represents a symbol identified by a unique number
    Symbol(u64),
    /// Represents an arbitrary-precision integer value
    Integer(BigInt),
    /// Represents a floating-point number
    Float(f64),
    /// Represents a single character
    Char(char),
    /// Represents a string value
    String(String),
}

impl Ast {
    /// Creates a new nil node.
    pub fn create_nil() -> Self {
        Ast(Tree::leaf(AstNode::Nil))
    }

    /// Checks if the node is nil.
    pub fn is_nil(&self) -> bool {
        matches!(self.0.data(), AstNode::Nil)
    }

    /// Creates a new pair node with the given head and tail.
    pub fn create_pair(head: &Self, tail: &Self) -> Self {
        Ast(Tree::node(AstNode::Pair, &[head.0.clone(), tail.0.clone()]))
    }

    /// Checks if the node is a pair.
    pub fn is_pair(&self) -> bool {
        matches!(self.0.data(), AstNode::Pair)
    }

    /// Returns the head and tail of a pair node, if this is a pair.
    pub fn get_pair(&self) -> Option<(Self, Self)> {
        if self.0.children().is_empty() {
            return None;
        }

        assert!(self.0.children().len() == 2);

        Some((
            Ast(self.0.children()[0].clone()),
            Ast(self.0.children()[1].clone()),
        ))
    }

    /// Creates a new symbol node with the given name.
    pub fn create_symbol(name: u64) -> Self {
        Ast(Tree::leaf(AstNode::Symbol(name)))
    }

    /// Checks if the node is a symbol.
    pub fn is_symbol(&self) -> bool {
        matches!(&*self.0.data(), AstNode::Symbol(..))
    }

    /// Returns the name of a symbol node, if this is a symbol.
    pub fn get_symbol(&self) -> Option<u64> {
        match &*self.0.data() {
            AstNode::Symbol(name) => Some(*name),
            _ => None,
        }
    }

    /// Creates a new integer node with the given value.
    pub fn create_integer(value: BigInt) -> Self {
        Ast(Tree::leaf(AstNode::Integer(value)))
    }

    /// Checks if the node is an integer.
    pub fn is_integer(&self) -> bool {
        matches!(&*self.0.data(), AstNode::Integer(..))
    }

    /// Returns the value of an integer node, if this is an integer.
    pub fn get_integer(&self) -> Option<&BigInt> {
        match &*self.0.data() {
            AstNode::Integer(value) => Some(value),
            _ => None,
        }
    }

    /// Creates a new float node with the given value.
    pub fn create_float(value: f64) -> Self {
        Ast(Tree::leaf(AstNode::Float(value)))
    }

    /// Checks if the node is a float.
    pub fn is_float(&self) -> bool {
        matches!(&*self.0.data(), AstNode::Float(..))
    }

    /// Returns the value of a float node, if this is a float.
    pub fn get_float(&self) -> Option<f64> {
        match &*self.0.data() {
            AstNode::Float(value) => Some(*value),
            _ => None,
        }
    }

    /// Creates a new character node with the given value.
    pub fn create_char(value: char) -> Self {
        Ast(Tree::leaf(AstNode::Char(value)))
    }

    /// Checks if the node is a character.
    pub fn is_char(&self) -> bool {
        matches!(&*self.0.data(), AstNode::Char(..))
    }

    /// Returns the value of a character node, if this is a character.
    pub fn get_char(&self) -> Option<char> {
        match &*self.0.data() {
            AstNode::Char(value) => Some(*value),
            _ => None,
        }
    }

    /// Creates a new string node with the given value.
    pub fn create_string(value: &str) -> Self {
        Ast(Tree::leaf(AstNode::String(value.to_string())))
    }

    /// Checks if the node is a string.
    pub fn is_string(&self) -> bool {
        matches!(&*self.0.data(), AstNode::String(..))
    }

    /// Returns the value of a string node, if this is a string.
    pub fn get_string(&self) -> Option<&str> {
        match &*self.0.data() {
            AstNode::String(value) => Some(value),
            _ => None,
        }
    }

    /// Visits all nodes in the AST using the provided visitor.
    ///
    /// # Arguments
    ///
    /// * `visitor` - The visitor implementation that will process each node
    /// * `prefix` - Whether to visit pair nodes before their children
    /// * `infix` - Whether to visit pair nodes between their children
    /// * `postfix` - Whether to visit pair nodes after their children
    pub fn visit<V: Visitor>(&self, visitor: &mut V, prefix: bool, infix: bool, postfix: bool) {
        let mut inner_visitor = InnerAstVisitor { visitor };
        self.0.visit(&mut inner_visitor, prefix, infix, postfix);
    }
}

struct InnerAstVisitor<'a, V: Visitor> {
    visitor: &'a mut V,
}

impl<'a, V> TreeVisitor<AstNode> for InnerAstVisitor<'a, V>
where
    V: Visitor,
{
    fn visit(&mut self, node: &Tree<AstNode>, traversal: TreeTraversal) {
        match &*node.data() {
            AstNode::Nil => self.visitor.visit_nil(SyntaxInfo::default()),
            AstNode::Symbol(symbol) => self.visitor.visit_symbol(*symbol, SyntaxInfo::default()),
            AstNode::Integer(value) => self.visitor.visit_integer(value, SyntaxInfo::default()),
            AstNode::Float(value) => self.visitor.visit_float(*value, SyntaxInfo::default()),
            AstNode::Char(value) => self.visitor.visit_char(*value, SyntaxInfo::default()),
            AstNode::String(value) => self.visitor.visit_string(value, SyntaxInfo::default()),

            AstNode::Pair => {
                let head = node.children()[0].clone();
                let tail = node.children()[1].clone();

                self.visitor.visit_pair(
                    &Ast(head),
                    &Ast(tail),
                    match traversal {
                        TreeTraversal::PreOrder => Traversal::Prefix,
                        TreeTraversal::InOrder => Traversal::Infix,
                        TreeTraversal::PostOrder => Traversal::Postfix,
                    },
                    SyntaxInfo::default(),
                );
            }
        }
    }
}

/// Specifies the type of traversal when visiting pair nodes.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Traversal {
    /// Visit a node before its children
    Prefix,
    /// Visit a node between its children
    Infix,
    /// Visit a node after its children
    Postfix,
}

/// Trait for implementing visitors that can process AST nodes.
#[allow(unused_variables)]
pub trait Visitor {
    /// Called when visiting a nil node
    fn visit_nil(&mut self, syntax_info: SyntaxInfo) {}
    /// Called when visiting a pair node
    fn visit_pair(
        &mut self,
        head: &Ast,
        tail: &Ast,
        traversal: Traversal,
        syntax_info: SyntaxInfo,
    ) {
    }
    /// Called when visiting a symbol node
    fn visit_symbol(&mut self, symbol: u64, syntax_info: SyntaxInfo) {}
    /// Called when visiting an integer node
    fn visit_integer(&mut self, value: &BigInt, syntax_info: SyntaxInfo) {}
    /// Called when visiting a float node
    fn visit_float(&mut self, value: f64, syntax_info: SyntaxInfo) {}
    /// Called when visiting a character node
    fn visit_char(&mut self, value: char, syntax_info: SyntaxInfo) {}
    /// Called when visiting a string node
    fn visit_string(&mut self, value: &str, syntax_info: SyntaxInfo) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Debug)]
    struct TestVisitor {
        visited: Vec<(AstNode, Traversal)>,
    }

    impl TestVisitor {
        fn new() -> Self {
            TestVisitor {
                visited: Vec::new(),
            }
        }
    }

    impl Visitor for TestVisitor {
        fn visit_nil(&mut self, _: SyntaxInfo) {
            self.visited.push((AstNode::Nil, Traversal::Prefix));
        }

        fn visit_pair(&mut self, _: &Ast, _: &Ast, traversal: Traversal, _: SyntaxInfo) {
            self.visited.push((AstNode::Pair, traversal));
        }

        fn visit_symbol(&mut self, symbol: u64, _: SyntaxInfo) {
            self.visited
                .push((AstNode::Symbol(symbol), Traversal::Prefix));
        }

        fn visit_integer(&mut self, value: &BigInt, _: SyntaxInfo) {
            self.visited
                .push((AstNode::Integer(value.clone()), Traversal::Prefix));
        }

        fn visit_float(&mut self, value: f64, _: SyntaxInfo) {
            self.visited
                .push((AstNode::Float(value), Traversal::Prefix));
        }

        fn visit_char(&mut self, value: char, _: SyntaxInfo) {
            self.visited.push((AstNode::Char(value), Traversal::Prefix));
        }

        fn visit_string(&mut self, value: &str, _: SyntaxInfo) {
            self.visited
                .push((AstNode::String(value.to_string()), Traversal::Prefix));
        }
    }

    #[test]
    fn test_visit_nil() {
        let ast = Ast::create_nil();
        let mut visitor = TestVisitor::new();
        ast.visit(&mut visitor, true, false, false);
        assert_eq!(visitor.visited, vec![(AstNode::Nil, Traversal::Prefix)]);
    }

    #[test]
    fn test_visit_symbol() {
        let ast = Ast::create_symbol(42);
        let mut visitor = TestVisitor::new();
        ast.visit(&mut visitor, true, false, false);
        assert_eq!(
            visitor.visited,
            vec![(AstNode::Symbol(42), Traversal::Prefix)]
        );
    }

    #[test]
    fn test_visit_integer() {
        let ast = Ast::create_integer(BigInt::from(123));
        let mut visitor = TestVisitor::new();
        ast.visit(&mut visitor, true, false, false);
        assert_eq!(
            visitor.visited,
            vec![(AstNode::Integer(BigInt::from(123)), Traversal::Prefix)]
        );
    }

    #[test]
    fn test_visit_float() {
        let ast = Ast::create_float(3.14);
        let mut visitor = TestVisitor::new();
        ast.visit(&mut visitor, true, false, false);
        assert_eq!(
            visitor.visited,
            vec![(AstNode::Float(3.14), Traversal::Prefix)]
        );
    }

    #[test]
    fn test_visit_char() {
        let ast = Ast::create_char('a');
        let mut visitor = TestVisitor::new();
        ast.visit(&mut visitor, true, false, false);
        assert_eq!(
            visitor.visited,
            vec![(AstNode::Char('a'), Traversal::Prefix)]
        );
    }

    #[test]
    fn test_visit_string() {
        let ast = Ast::create_string("hello");
        let mut visitor = TestVisitor::new();
        ast.visit(&mut visitor, true, false, false);
        assert_eq!(
            visitor.visited,
            vec![(AstNode::String("hello".to_string()), Traversal::Prefix)]
        );
    }

    #[test]
    fn test_visit_pair() {
        let head = Ast::create_symbol(1);
        let tail = Ast::create_symbol(2);
        let pair = Ast::create_pair(&head, &tail);
        let mut visitor = TestVisitor::new();
        pair.visit(&mut visitor, true, false, false);
        assert_eq!(
            visitor.visited,
            vec![
                (AstNode::Pair, Traversal::Prefix),
                (AstNode::Symbol(1), Traversal::Prefix),
                (AstNode::Symbol(2), Traversal::Prefix),
            ]
        );
    }
}
