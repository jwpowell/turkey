use std::fmt::Debug;
use std::rc::Rc;

use super::nfa::Nfa;

#[derive(Clone)]
pub struct Regex {
    pub(crate) node: Rc<RegexNode>,
    nullable: bool,
}

pub enum RegexNode {
    Empty,
    Epsilon,
    Range(char, char),
    Concat(Regex, Regex),
    Union(Regex, Regex),
    Intersect(Regex, Regex),
    Star(Regex),
    Not(Regex),
}

impl Debug for Regex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &*self.node {
            RegexNode::Empty => write!(f, "∅"),
            RegexNode::Epsilon => write!(f, "ε"),
            RegexNode::Range(lo, hi) => write!(f, "(range {:?} {:?})", lo, hi),
            RegexNode::Concat(a, b) => write!(f, "(concat {:?} {:?})", a, b),
            RegexNode::Union(a, b) => write!(f, "(union {:?} {:?})", a, b),
            RegexNode::Intersect(a, b) => write!(f, "(intersect {:?} {:?})", a, b),
            RegexNode::Star(a) => write!(f, "(star {:?})", a),
            RegexNode::Not(a) => write!(f, "(not {:?})", a),
        }
    }
}

pub fn empty() -> Regex {
    Regex::empty()
}

pub fn epsilon() -> Regex {
    Regex::epsilon()
}

pub fn any() -> Regex {
    Regex::any()
}

pub fn char(c: char) -> Regex {
    Regex::char(c)
}

pub fn range(lo: char, hi: char) -> Regex {
    Regex::range(lo, hi)
}

pub fn one_of(chars: &str) -> Regex {
    Regex::one_of(chars)
}

pub fn none_of(chars: &str) -> Regex {
    Regex::none_of(chars)
}

impl Regex {
    pub fn is_empty(&self) -> bool {
        matches!(*self.node, RegexNode::Empty)
    }

    pub fn is_epsilon(&self) -> bool {
        matches!(*self.node, RegexNode::Epsilon)
    }

    pub fn empty() -> Self {
        Self {
            node: Rc::new(RegexNode::Empty),
            nullable: false,
        }
    }

    pub fn epsilon() -> Self {
        Self {
            node: Rc::new(RegexNode::Epsilon),
            nullable: true,
        }
    }

    pub fn any() -> Self {
        Self::range(char::MIN, char::MAX)
    }

    pub fn char(c: char) -> Self {
        Self::range(c, c)
    }

    pub fn range(lo: char, hi: char) -> Self {
        Self {
            node: Rc::new(RegexNode::Range(lo, hi)),
            nullable: false,
        }
    }

    pub fn concat(&self, other: &Self) -> Self {
        Self {
            node: Rc::new(RegexNode::Concat(self.clone(), other.clone())),
            nullable: self.nullable && other.nullable,
        }
    }

    pub fn union(&self, other: &Self) -> Self {
        Self {
            node: Rc::new(RegexNode::Union(self.clone(), other.clone())),
            nullable: self.nullable || other.nullable,
        }
    }

    pub fn intersect(&self, other: &Self) -> Self {
        Self {
            node: Rc::new(RegexNode::Intersect(self.clone(), other.clone())),
            nullable: self.nullable && other.nullable,
        }
    }

    pub fn star(&self) -> Self {
        Self {
            node: Rc::new(RegexNode::Star(self.clone())),
            nullable: true,
        }
    }

    pub fn plus(&self) -> Self {
        self.concat(&self.star())
    }

    pub fn opt(&self) -> Self {
        self.union(&Self::epsilon())
    }

    pub fn not(&self) -> Self {
        Self {
            node: Rc::new(RegexNode::Not(self.clone())),
            nullable: !self.nullable,
        }
    }

    pub fn one_of(chars: &str) -> Self {
        let mut r = Self::empty();

        for c in chars.chars() {
            r = r.union(&Self::char(c));
        }

        r
    }

    pub fn none_of(chars: &str) -> Self {
        Self::one_of(chars).not()
    }

    pub fn nfa<T: Clone>(&self, output: T) -> Nfa<T> {
        todo!()
    }
}
