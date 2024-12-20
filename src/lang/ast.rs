/// Abstract syntax tree (AST) module for representing and manipulating Lisp-like expressions.
///
/// This module provides the core data structures and traits for working with AST nodes,
/// including support for different types of values (nil, pairs, symbols, numbers, etc.)
/// and tree traversal operations.
use crate::utils::graph::*;

use num::BigInt;

/// Represents an AST node.
#[derive(Debug, Clone)]
pub struct Ast {}

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
        todo!()
    }

    /// Checks if the node is nil.
    pub fn is_nil(&self) -> bool {
        todo!()
    }

    /// Creates a new pair node with the given head and tail.
    pub fn create_pair(head: &Self, tail: &Self) -> Self {
        todo!()
    }

    /// Checks if the node is a pair.
    pub fn is_pair(&self) -> bool {
        todo!()
    }

    /// Returns the head and tail of a pair node, if this is a pair.
    pub fn get_pair(&self) -> Option<(Self, Self)> {
        todo!()
    }

    /// Creates a new symbol node with the given name.
    pub fn create_symbol(name: u64) -> Self {
        todo!()
    }

    /// Checks if the node is a symbol.
    pub fn is_symbol(&self) -> bool {
        todo!()
    }

    /// Returns the name of a symbol node, if this is a symbol.
    pub fn get_symbol(&self) -> Option<u64> {
        todo!()
    }

    /// Creates a new integer node with the given value.
    pub fn create_integer(value: BigInt) -> Self {
        todo!()
    }

    /// Checks if the node is an integer.
    pub fn is_integer(&self) -> bool {
        todo!()
    }

    /// Returns the value of an integer node, if this is an integer.
    pub fn get_integer(&self) -> Option<&BigInt> {
        todo!()
    }

    /// Creates a new float node with the given value.
    pub fn create_float(value: f64) -> Self {
        todo!()
    }

    /// Checks if the node is a float.
    pub fn is_float(&self) -> bool {
        todo!()
    }

    /// Returns the value of a float node, if this is a float.
    pub fn get_float(&self) -> Option<f64> {
        todo!()
    }

    /// Creates a new character node with the given value.
    pub fn create_char(value: char) -> Self {
        todo!()
    }

    /// Checks if the node is a character.
    pub fn is_char(&self) -> bool {
        todo!()
    }

    /// Returns the value of a character node, if this is a character.
    pub fn get_char(&self) -> Option<char> {
        todo!()
    }

    /// Creates a new string node with the given value.
    pub fn create_string(value: &str) -> Self {
        todo!()
    }

    /// Checks if the node is a string.
    pub fn is_string(&self) -> bool {
        todo!()
    }

    /// Returns the value of a string node, if this is a string.
    pub fn get_string(&self) -> Option<&str> {
        todo!()
    }
}
