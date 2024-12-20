/// Abstract syntax tree (AST) module for representing and manipulating Lisp-like expressions.
///
/// This module provides the core data structures and traits for working with AST nodes,
/// including support for different types of values (nil, pairs, symbols, numbers, etc.)
/// and tree traversal operations.
use crate::utils::graph::*;

use num::BigInt;

/// Represents an AST node.
#[derive(Debug, Clone)]
pub struct Ast {
    graph: Graph<AstNode, ()>,
    root: u64,
}

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
    pub fn create_nil(&mut self) -> u64 {
        self.graph.add_vertex(AstNode::Nil)
    }

    /// Checks if the node is nil.
    pub fn is_nil(&self, vertex: u64) -> bool {
        if let Some(vertex) = self.graph.get_vertex(vertex) {
            matches!(vertex.data(), AstNode::Nil)
        } else {
            false
        }
    }

    /// Creates a new pair node with the given head and tail.
    pub fn create_pair(&mut self, head: u64, tail: u64) -> u64 {
        let id = self.graph.add_vertex(AstNode::Pair);
        self.graph.add_edge(id, head, ());
        self.graph.add_edge(id, tail, ());
        id
    }

    /// Checks if the node is a pair.
    pub fn is_pair(&self, id: u64) -> bool {
        if let Some(vertex) = self.graph.get_vertex(id) {
            matches!(vertex.data(), AstNode::Pair)
        } else {
            false
        }
    }

    /// Returns the head and tail of a pair node, if this is a pair.
    pub fn get_pair(&self, id: u64) -> Option<(u64, u64)> {
        if !self.is_pair(id) {
            return None;
        }

        let edges = self.graph.get_edges(id)?;

        debug_assert!(edges.len() == 2);

        let head = edges[0].target;
        let tail = edges[1].target;

        Some((head, tail))
    }

    /// Creates a new symbol node with the given name.
    pub fn create_symbol(&mut self, name: u64) -> u64 {
        self.graph.add_vertex(AstNode::Symbol(name))
    }

    /// Checks if the node is a symbol.
    pub fn is_symbol(&self, id: u64) -> bool {
        if let Some(vertex) = self.graph.get_vertex(id) {
            matches!(vertex.data(), AstNode::Symbol(_))
        } else {
            false
        }
    }

    /// Returns the name of a symbol node, if this is a symbol.
    pub fn get_symbol(&self, id: u64) -> Option<u64> {
        let vertex = self.graph.get_vertex(id)?;

        if let AstNode::Symbol(name) = vertex.data() {
            Some(*name)
        } else {
            None
        }
    }

    /// Creates a new integer node with the given value.
    pub fn create_integer(&mut self, value: BigInt) -> u64 {
        self.graph.add_vertex(AstNode::Integer(value))
    }

    /// Checks if the node is an integer.
    pub fn is_integer(&self, id: u64) -> bool {
        self.get_integer(id).is_some()
    }

    /// Returns the value of an integer node, if this is an integer.
    pub fn get_integer(&self, id: u64) -> Option<&BigInt> {
        let vertex = self.graph.get_vertex(id)?;

        if let AstNode::Integer(value) = vertex.data() {
            Some(value)
        } else {
            None
        }
    }

    /// Creates a new float node with the given value.
    pub fn create_float(&mut self, value: f64) -> u64 {
        self.graph.add_vertex(AstNode::Float(value))
    }

    /// Checks if the node is a float.
    pub fn is_float(&self, id: u64) -> bool {
        self.get_float(id).is_some()
    }

    /// Returns the value of a float node, if this is a float.
    pub fn get_float(&self, id: u64) -> Option<f64> {
        let vertex = self.graph.get_vertex(id)?;

        if let AstNode::Float(value) = vertex.data() {
            Some(*value)
        } else {
            None
        }
    }

    /// Creates a new character node with the given value.
    pub fn create_char(&mut self, value: char) -> u64 {
        self.graph.add_vertex(AstNode::Char(value))
    }

    /// Checks if the node is a character.
    pub fn is_char(&self, id: u64) -> bool {
        self.get_char(id).is_some()
    }

    /// Returns the value of a character node, if this is a character.
    pub fn get_char(&self, id: u64) -> Option<char> {
        let vertex = self.graph.get_vertex(id)?;

        if let AstNode::Char(value) = vertex.data() {
            Some(*value)
        } else {
            None
        }
    }

    /// Creates a new string node with the given value.
    pub fn create_string(&mut self, value: &str) -> u64 {
        self.graph.add_vertex(AstNode::String(value.to_string()))
    }

    /// Checks if the node is a string.
    pub fn is_string(&self, id: u64) -> bool {
        self.get_string(id).is_some()
    }

    /// Returns the value of a string node, if this is a string.
    pub fn get_string(&self, id: u64) -> Option<&str> {
        let vertex = self.graph.get_vertex(id)?;

        if let AstNode::String(value) = vertex.data() {
            Some(value)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_nil() {
        let mut ast = Ast {
            graph: Graph::new(),
            root: 0,
        };
        let nil_id = ast.create_nil();
        assert!(ast.is_nil(nil_id));
    }

    #[test]
    fn test_create_pair() {
        let mut ast = Ast {
            graph: Graph::new(),
            root: 0,
        };
        let head = ast.create_nil();
        let tail = ast.create_nil();
        let pair_id = ast.create_pair(head, tail);
        assert!(ast.is_pair(pair_id));
        assert_eq!(ast.get_pair(pair_id), Some((head, tail)));
    }

    #[test]
    fn test_create_symbol() {
        let mut ast = Ast {
            graph: Graph::new(),
            root: 0,
        };
        let symbol_id = ast.create_symbol(42);
        assert!(ast.is_symbol(symbol_id));
        assert_eq!(ast.get_symbol(symbol_id), Some(42));
    }

    #[test]
    fn test_create_integer() {
        let mut ast = Ast {
            graph: Graph::new(),
            root: 0,
        };
        let value = BigInt::from(123);
        let int_id = ast.create_integer(value.clone());
        assert!(ast.is_integer(int_id));
        assert_eq!(ast.get_integer(int_id), Some(&value));
    }

    #[test]
    fn test_create_float() {
        let mut ast = Ast {
            graph: Graph::new(),
            root: 0,
        };
        let float_id = ast.create_float(3.14);
        assert!(ast.is_float(float_id));
        assert_eq!(ast.get_float(float_id), Some(3.14));
    }

    #[test]
    fn test_create_char() {
        let mut ast = Ast {
            graph: Graph::new(),
            root: 0,
        };
        let char_id = ast.create_char('a');
        assert!(ast.is_char(char_id));
        assert_eq!(ast.get_char(char_id), Some('a'));
    }

    #[test]
    fn test_create_string() {
        let mut ast = Ast {
            graph: Graph::new(),
            root: 0,
        };
        let string_id = ast.create_string("hello");
        assert!(ast.is_string(string_id));
        assert_eq!(ast.get_string(string_id), Some("hello"));
    }

    #[test]
    fn test_is_nil() {
        let mut ast = Ast {
            graph: Graph::new(),
            root: 0,
        };
        let nil_id = ast.create_nil();
        assert!(ast.is_nil(nil_id));
        let symbol_id = ast.create_symbol(42);
        assert!(!ast.is_nil(symbol_id));
    }

    #[test]
    fn test_is_pair() {
        let mut ast = Ast {
            graph: Graph::new(),
            root: 0,
        };
        let head = ast.create_nil();
        let tail = ast.create_nil();
        let pair_id = ast.create_pair(head, tail);
        assert!(ast.is_pair(pair_id));
        let symbol_id = ast.create_symbol(42);
        assert!(!ast.is_pair(symbol_id));
    }

    #[test]
    fn test_is_symbol() {
        let mut ast = Ast {
            graph: Graph::new(),
            root: 0,
        };
        let symbol_id = ast.create_symbol(42);
        assert!(ast.is_symbol(symbol_id));
        let nil_id = ast.create_nil();
        assert!(!ast.is_symbol(nil_id));
    }

    #[test]
    fn test_is_integer() {
        let mut ast = Ast {
            graph: Graph::new(),
            root: 0,
        };
        let value = BigInt::from(123);
        let int_id = ast.create_integer(value.clone());
        assert!(ast.is_integer(int_id));
        let nil_id = ast.create_nil();
        assert!(!ast.is_integer(nil_id));
    }

    #[test]
    fn test_is_float() {
        let mut ast = Ast {
            graph: Graph::new(),
            root: 0,
        };
        let float_id = ast.create_float(3.14);
        assert!(ast.is_float(float_id));
        let nil_id = ast.create_nil();
        assert!(!ast.is_float(nil_id));
    }

    #[test]
    fn test_is_char() {
        let mut ast = Ast {
            graph: Graph::new(),
            root: 0,
        };
        let char_id = ast.create_char('a');
        assert!(ast.is_char(char_id));
        let nil_id = ast.create_nil();
        assert!(!ast.is_char(nil_id));
    }

    #[test]
    fn test_is_string() {
        let mut ast = Ast {
            graph: Graph::new(),
            root: 0,
        };
        let string_id = ast.create_string("hello");
        assert!(ast.is_string(string_id));
        let nil_id = ast.create_nil();
        assert!(!ast.is_string(nil_id));
    }

    #[test]
    fn test_get_pair_invalid() {
        let mut ast = Ast {
            graph: Graph::new(),
            root: 0,
        };
        let symbol_id = ast.create_symbol(42);
        assert!(ast.get_pair(symbol_id).is_none());
    }

    #[test]
    fn test_get_symbol_invalid() {
        let mut ast = Ast {
            graph: Graph::new(),
            root: 0,
        };
        let nil_id = ast.create_nil();
        assert!(ast.get_symbol(nil_id).is_none());
    }

    #[test]
    fn test_get_integer_invalid() {
        let mut ast = Ast {
            graph: Graph::new(),
            root: 0,
        };
        let nil_id = ast.create_nil();
        assert!(ast.get_integer(nil_id).is_none());
    }

    #[test]
    fn test_get_float_invalid() {
        let mut ast = Ast {
            graph: Graph::new(),
            root: 0,
        };
        let nil_id = ast.create_nil();
        assert!(ast.get_float(nil_id).is_none());
    }

    #[test]
    fn test_get_char_invalid() {
        let mut ast = Ast {
            graph: Graph::new(),
            root: 0,
        };
        let nil_id = ast.create_nil();
        assert!(ast.get_char(nil_id).is_none());
    }

    #[test]
    fn test_get_string_invalid() {
        let mut ast = Ast {
            graph: Graph::new(),
            root: 0,
        };
        let nil_id = ast.create_nil();
        assert!(ast.get_string(nil_id).is_none());
    }

    #[test]
    fn test_create_and_check_nodes() {
        let mut ast = Ast {
            graph: Graph::new(),
            root: 0,
        };

        // Test nil node
        let nil_id = ast.create_nil();
        assert!(ast.is_nil(nil_id));
        assert!(!ast.is_pair(nil_id));
        assert!(!ast.is_symbol(nil_id));
        assert!(!ast.is_integer(nil_id));
        assert!(!ast.is_float(nil_id));
        assert!(!ast.is_char(nil_id));
        assert!(!ast.is_string(nil_id));

        // Test pair node
        let head = ast.create_nil();
        let tail = ast.create_nil();
        let pair_id = ast.create_pair(head, tail);
        assert!(ast.is_pair(pair_id));
        assert_eq!(ast.get_pair(pair_id), Some((head, tail)));
        assert!(!ast.is_nil(pair_id));
        assert!(!ast.is_symbol(pair_id));
        assert!(!ast.is_integer(pair_id));
        assert!(!ast.is_float(pair_id));
        assert!(!ast.is_char(pair_id));
        assert!(!ast.is_string(pair_id));

        // Test symbol node
        let symbol_id = ast.create_symbol(42);
        assert!(ast.is_symbol(symbol_id));
        assert_eq!(ast.get_symbol(symbol_id), Some(42));
        assert!(!ast.is_nil(symbol_id));
        assert!(!ast.is_pair(symbol_id));
        assert!(!ast.is_integer(symbol_id));
        assert!(!ast.is_float(symbol_id));
        assert!(!ast.is_char(symbol_id));
        assert!(!ast.is_string(symbol_id));

        // Test integer node
        let value = BigInt::from(123);
        let int_id = ast.create_integer(value.clone());
        assert!(ast.is_integer(int_id));
        assert_eq!(ast.get_integer(int_id), Some(&value));
        assert!(!ast.is_nil(int_id));
        assert!(!ast.is_pair(int_id));
        assert!(!ast.is_symbol(int_id));
        assert!(!ast.is_float(int_id));
        assert!(!ast.is_char(int_id));
        assert!(!ast.is_string(int_id));

        // Test float node
        let float_id = ast.create_float(3.14);
        assert!(ast.is_float(float_id));
        assert_eq!(ast.get_float(float_id), Some(3.14));
        assert!(!ast.is_nil(float_id));
        assert!(!ast.is_pair(float_id));
        assert!(!ast.is_symbol(float_id));
        assert!(!ast.is_integer(float_id));
        assert!(!ast.is_char(float_id));
        assert!(!ast.is_string(float_id));

        // Test char node
        let char_id = ast.create_char('a');
        assert!(ast.is_char(char_id));
        assert_eq!(ast.get_char(char_id), Some('a'));
        assert!(!ast.is_nil(char_id));
        assert!(!ast.is_pair(char_id));
        assert!(!ast.is_symbol(char_id));
        assert!(!ast.is_integer(char_id));
        assert!(!ast.is_float(char_id));
        assert!(!ast.is_string(char_id));

        // Test string node
        let string_id = ast.create_string("hello");
        assert!(ast.is_string(string_id));
        assert_eq!(ast.get_string(string_id), Some("hello"));
        assert!(!ast.is_nil(string_id));
        assert!(!ast.is_pair(string_id));
        assert!(!ast.is_symbol(string_id));
        assert!(!ast.is_integer(string_id));
        assert!(!ast.is_float(string_id));
        assert!(!ast.is_char(string_id));
    }
}
