/// Abstract syntax tree (AST) module for representing and manipulating Lisp-like expressions.
///
/// This module provides the core data structures and traits for working with AST nodes,
/// including support for different types of values (nil, pairs, symbols, numbers, etc.)
/// and tree traversal operations.
use std::rc::Rc;

use num::BigInt;

/// Represents an AST node.
#[derive(Debug, Clone)]
pub struct Ast(Rc<AstNode>);

#[derive(Debug, Clone, Copy, Default)]
pub struct SyntaxInfo {}

/// Represents the different types of nodes that can exist in the AST.
#[derive(Debug, Clone)]
enum AstNode {
    /// Represents a nil value
    Nil,
    /// Represents a pair of AST nodes (head, tail)
    Pair(Ast, Ast),
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
        Ast(Rc::new(AstNode::Nil))
    }

    /// Checks if the node is nil.
    pub fn is_nil(&self) -> bool {
        matches!(&*self.0, AstNode::Nil)
    }

    /// Creates a new pair node with the given head and tail.
    pub fn create_pair(head: &Self, tail: &Self) -> Self {
        Ast(Rc::new(AstNode::Pair(head.clone(), tail.clone())))
    }

    /// Checks if the node is a pair.
    pub fn is_pair(&self) -> bool {
        matches!(&*self.0, AstNode::Pair(..))
    }

    /// Returns the head and tail of a pair node, if this is a pair.
    pub fn get_pair(&self) -> Option<(&Self, &Self)> {
        match &*self.0 {
            AstNode::Pair(head, tail) => Some((head, tail)),
            _ => None,
        }
    }

    /// Creates a new symbol node with the given name.
    pub fn create_symbol(name: u64) -> Self {
        Ast(Rc::new(AstNode::Symbol(name)))
    }

    /// Checks if the node is a symbol.
    pub fn is_symbol(&self) -> bool {
        matches!(&*self.0, AstNode::Symbol(..))
    }

    /// Returns the name of a symbol node, if this is a symbol.
    pub fn get_symbol(&self) -> Option<u64> {
        match &*self.0 {
            AstNode::Symbol(name) => Some(*name),
            _ => None,
        }
    }

    /// Creates a new integer node with the given value.
    pub fn create_integer(value: BigInt) -> Self {
        Ast(Rc::new(AstNode::Integer(value)))
    }

    /// Checks if the node is an integer.
    pub fn is_integer(&self) -> bool {
        matches!(&*self.0, AstNode::Integer(..))
    }

    /// Returns the value of an integer node, if this is an integer.
    pub fn get_integer(&self) -> Option<&BigInt> {
        match &*self.0 {
            AstNode::Integer(value) => Some(value),
            _ => None,
        }
    }

    /// Creates a new float node with the given value.
    pub fn create_float(value: f64) -> Self {
        Ast(Rc::new(AstNode::Float(value)))
    }

    /// Checks if the node is a float.
    pub fn is_float(&self) -> bool {
        matches!(&*self.0, AstNode::Float(..))
    }

    /// Returns the value of a float node, if this is a float.
    pub fn get_float(&self) -> Option<f64> {
        match &*self.0 {
            AstNode::Float(value) => Some(*value),
            _ => None,
        }
    }

    /// Creates a new character node with the given value.
    pub fn create_char(value: char) -> Self {
        Ast(Rc::new(AstNode::Char(value)))
    }

    /// Checks if the node is a character.
    pub fn is_char(&self) -> bool {
        matches!(&*self.0, AstNode::Char(..))
    }

    /// Returns the value of a character node, if this is a character.
    pub fn get_char(&self) -> Option<char> {
        match &*self.0 {
            AstNode::Char(value) => Some(*value),
            _ => None,
        }
    }

    /// Creates a new string node with the given value.
    pub fn create_string(value: &str) -> Self {
        Ast(Rc::new(AstNode::String(value.to_string())))
    }

    /// Checks if the node is a string.
    pub fn is_string(&self) -> bool {
        matches!(&*self.0, AstNode::String(..))
    }

    /// Returns the value of a string node, if this is a string.
    pub fn get_string(&self) -> Option<&str> {
        match &*self.0 {
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
        let mut stack = vec![(self, None)];

        while let Some((node, traversal)) = stack.pop() {
            match &*node.0 {
                AstNode::Nil => visitor.visit_nil(SyntaxInfo::default()),
                AstNode::Symbol(symbol) => visitor.visit_symbol(*symbol, SyntaxInfo::default()),
                AstNode::Integer(value) => visitor.visit_integer(value, SyntaxInfo::default()),
                AstNode::Float(value) => visitor.visit_float(*value, SyntaxInfo::default()),
                AstNode::Char(value) => visitor.visit_char(*value, SyntaxInfo::default()),
                AstNode::String(value) => visitor.visit_string(value, SyntaxInfo::default()),

                AstNode::Pair(head, tail) => {
                    if traversal.is_some() {
                        visitor.visit_pair(head, tail, traversal.unwrap(), SyntaxInfo::default());
                        continue;
                    }

                    if postfix {
                        stack.push((self, Some(Traversal::Postfix)));
                    }

                    stack.push((tail, None));

                    if infix {
                        stack.push((self, Some(Traversal::Infix)));
                    }

                    stack.push((head, None));

                    if prefix {
                        stack.push((self, Some(Traversal::Prefix)));
                    }
                }
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

    #[test]
    fn test_visit_prefix() {
        let a = Ast::create_string("a");
        let b = Ast::create_string("b");
        let c = Ast::create_string("c");
        let d = Ast::create_string("d");

        let ab = Ast::create_pair(&a, &b);
        let cd = Ast::create_pair(&c, &d);
        let abcd = Ast::create_pair(&ab, &cd);

        struct TestVisitor {
            count: u64,
            accumulator: String,
        }

        impl Visitor for TestVisitor {
            fn visit_pair(&mut self, _: &Ast, _: &Ast, traversal: Traversal, _: SyntaxInfo) {
                assert_eq!(traversal, Traversal::Prefix);
                self.accumulator
                    .push_str(format!("{}", self.count).as_str());
                self.count += 1;
            }

            fn visit_string(&mut self, value: &str, _: SyntaxInfo) {
                self.accumulator.push_str(value);
            }
        }

        let mut visitor = TestVisitor {
            count: 0,
            accumulator: String::new(),
        };

        abcd.visit(&mut visitor, true, false, false);
        assert_eq!(visitor.accumulator, "01ab2cd");
    }

    #[test]
    fn test_visit_infix() {
        let a = Ast::create_string("a");
        let b = Ast::create_string("b");
        let c = Ast::create_string("c");
        let d = Ast::create_string("d");

        let ab = Ast::create_pair(&a, &b);
        let cd = Ast::create_pair(&c, &d);
        let abcd = Ast::create_pair(&ab, &cd);

        struct TestVisitor {
            count: u64,
            accumulator: String,
        }

        impl Visitor for TestVisitor {
            fn visit_pair(&mut self, _: &Ast, _: &Ast, traversal: Traversal, _: SyntaxInfo) {
                assert_eq!(traversal, Traversal::Infix);
                self.accumulator
                    .push_str(format!("{}", self.count).as_str());
                self.count += 1;
            }

            fn visit_string(&mut self, value: &str, _: SyntaxInfo) {
                self.accumulator.push_str(value);
            }
        }

        let mut visitor = TestVisitor {
            count: 0,
            accumulator: String::new(),
        };

        abcd.visit(&mut visitor, false, true, false);
        assert_eq!(visitor.accumulator, "a0b1c2d");
    }

    #[test]
    fn test_visit_postfix() {
        let a = Ast::create_string("a");
        let b = Ast::create_string("b");
        let c = Ast::create_string("c");
        let d = Ast::create_string("d");

        let ab = Ast::create_pair(&a, &b);
        let cd = Ast::create_pair(&c, &d);
        let abcd = Ast::create_pair(&ab, &cd);

        struct TestVisitor {
            count: u64,
            accumulator: String,
        }

        impl Visitor for TestVisitor {
            fn visit_pair(&mut self, _: &Ast, _: &Ast, traversal: Traversal, _: SyntaxInfo) {
                assert_eq!(traversal, Traversal::Postfix);
                self.accumulator
                    .push_str(format!("{}", self.count).as_str());
                self.count += 1;
            }

            fn visit_string(&mut self, value: &str, _: SyntaxInfo) {
                self.accumulator.push_str(value);
            }
        }

        let mut visitor = TestVisitor {
            count: 0,
            accumulator: String::new(),
        };

        abcd.visit(&mut visitor, false, false, true);
        assert_eq!(visitor.accumulator, "ab0cd12");
    }

    #[test]
    fn test_visit_all() {
        let a = Ast::create_string("a");
        let b = Ast::create_string("b");
        let c = Ast::create_string("c");
        let d = Ast::create_string("d");

        let ab = Ast::create_pair(&a, &b);
        let cd = Ast::create_pair(&c, &d);
        let abcd = Ast::create_pair(&ab, &cd);

        struct TestVisitor {
            count: u64,
            accumulator: String,
        }

        impl Visitor for TestVisitor {
            fn visit_pair(&mut self, _: &Ast, _: &Ast, _: Traversal, _: SyntaxInfo) {
                self.accumulator
                    .push_str(format!("{}", self.count).as_str());
                self.count += 1;
            }

            fn visit_string(&mut self, value: &str, _: SyntaxInfo) {
                self.accumulator.push_str(value);
            }
        }

        let mut visitor = TestVisitor {
            count: 0,
            accumulator: String::new(),
        };

        abcd.visit(&mut visitor, true, true, true);
        assert_eq!(visitor.accumulator, "01a2b345c6d78");
    }
}
