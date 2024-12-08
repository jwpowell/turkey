use std::collections::{HashMap, HashSet};
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Regex {
    node: Rc<RegexNode>,
    nullable: bool,
}

#[derive(Debug, Clone)]
enum RegexNode {
    Empty,
    Epsilon,
    Symbol(char),
    Range(char, char),
    Concat(Regex, Regex),
    Union(Regex, Regex),
    Intersect(Regex, Regex),
    Star(Regex),
    Compl(Regex),
}

impl Regex {
    pub fn empty() -> Self {
        Regex {
            node: Rc::new(RegexNode::Empty),
            nullable: false,
        }
    }

    pub fn epsilon() -> Self {
        Regex {
            node: Rc::new(RegexNode::Epsilon),
            nullable: true,
        }
    }

    pub fn symbol(c: char) -> Self {
        Regex {
            node: Rc::new(RegexNode::Symbol(c)),
            nullable: false,
        }
    }

    pub fn range(from: char, to: char) -> Self {
        Regex {
            node: Rc::new(RegexNode::Range(from, to)),
            nullable: false,
        }
    }

    pub fn concat(&self, other: &Self) -> Self {
        if self.is_empty() || other.is_epsilon() {
            return self.clone();
        }

        if other.is_empty() || self.is_epsilon() {
            return other.clone();
        }

        Regex {
            node: Rc::new(RegexNode::Concat(self.clone(), other.clone())),
            nullable: self.nullable && other.nullable,
        }
    }

    pub fn union(&self, other: &Self) -> Self {
        if self.is_empty() {
            return other.clone();
        }

        if other.is_empty() {
            return self.clone();
        }

        Regex {
            node: Rc::new(RegexNode::Union(self.clone(), other.clone())),
            nullable: self.nullable || other.nullable,
        }
    }

    pub fn first(&self, ranges: &mut Vec<(char, char)>) {
        match &*self.node {
            RegexNode::Empty => {}
            RegexNode::Epsilon => {}
            RegexNode::Symbol(c) => {
                ranges.push((*c, *c));
            }
            RegexNode::Range(from, to) => {
                ranges.push((*from, *to));
            }
            RegexNode::Concat(r1, r2) => {
                r1.first(ranges);
                if r1.is_nullable() {
                    r2.first(ranges);
                }
            }
            RegexNode::Union(r1, r2) => {
                r1.first(ranges);
                r2.first(ranges);
            }
            RegexNode::Intersect(r1, r2) => {
                r1.first(ranges);
                r2.first(ranges);
            }
            RegexNode::Star(r) => {
                r.first(ranges);
            }
            RegexNode::Compl(r) => {
                r.first(ranges);
            }
        }
    }

    pub fn intersect(&self, other: &Self) -> Self {
        Regex {
            node: Rc::new(RegexNode::Intersect(self.clone(), other.clone())),
            nullable: self.nullable && other.nullable,
        }
    }

    pub fn star(&self) -> Self {
        match &*self.node {
            RegexNode::Empty | RegexNode::Epsilon => Regex::epsilon(),
            RegexNode::Star(_) => self.clone(),
            _ => Regex {
                node: Rc::new(RegexNode::Star(self.clone())),
                nullable: true,
            },
        }
    }

    pub fn compl(&self) -> Self {
        Regex {
            node: Rc::new(RegexNode::Compl(self.clone())),
            nullable: !self.nullable,
        }
    }

    pub fn plus(&self) -> Self {
        self.concat(&self.star())
    }

    pub fn any_of(cs: &str) -> Self {
        let mut re = Regex::empty();
        for c in cs.chars() {
            re = re.union(&Regex::symbol(c));
        }
        re
    }

    pub fn seq(cs: &str) -> Self {
        let mut re = Regex::epsilon();
        for c in cs.chars() {
            re = re.concat(&Regex::symbol(c));
        }
        re
    }

    fn v(&self) -> Self {
        if self.nullable {
            return Regex::epsilon();
        } else {
            return Regex::empty();
        }
    }

    pub fn derivative(&self, c: char) -> Self {
        match &*self.node {
            RegexNode::Empty => Regex::empty(),
            RegexNode::Epsilon => Regex::empty(),
            RegexNode::Symbol(s) => {
                if *s == c {
                    return Regex::epsilon();
                } else {
                    return Regex::empty();
                }
            }
            RegexNode::Range(from, to) => {
                if *from <= c && c <= *to {
                    return Regex::epsilon();
                } else {
                    return Regex::empty();
                }
            }
            RegexNode::Concat(r1, r2) => {
                return r1
                    .derivative(c)
                    .concat(r2)
                    .union(&r1.v().concat(&r2.derivative(c)));
            }
            RegexNode::Union(r1, r2) => {
                return r1.derivative(c).union(&r2.derivative(c));
            }
            RegexNode::Intersect(r1, r2) => {
                return r1.derivative(c).intersect(&r2.derivative(c));
            }
            RegexNode::Star(r) => {
                return r.derivative(c).concat(self);
            }
            RegexNode::Compl(r) => {
                return r.derivative(c).compl();
            }
        }
    }

    pub fn is_nullable(&self) -> bool {
        self.nullable
    }

    pub fn is_epsilon(&self) -> bool {
        matches!(&*self.node, RegexNode::Epsilon)
    }

    pub fn is_empty(&self) -> bool {
        matches!(&*self.node, RegexNode::Empty)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let re = Regex::empty();
        assert!(re.is_empty());
        assert!(!re.is_nullable());
    }

    #[test]
    fn test_epsilon() {
        let re = Regex::epsilon();
        assert!(re.is_epsilon());
        assert!(re.is_nullable());
    }

    #[test]
    fn test_symbol() {
        let re = Regex::symbol('a');
        assert!(!re.is_empty());
        assert!(!re.is_nullable());
    }

    #[test]
    fn test_range() {
        let re = Regex::range('a', 'z');
        assert!(!re.is_empty());
        assert!(!re.is_nullable());
    }

    #[test]
    fn test_concat() {
        let re1 = Regex::symbol('a');
        let re2 = Regex::symbol('b');
        let re = re1.concat(&re2);
        assert!(!re.is_empty());
        assert!(!re.is_nullable());
    }

    #[test]
    fn test_union() {
        let re1 = Regex::symbol('a');
        let re2 = Regex::symbol('b');
        let re = re1.union(&re2);
        assert!(!re.is_empty());
        assert!(!re.is_nullable());
    }

    #[test]
    fn test_star() {
        let re = Regex::symbol('a').star();
        assert!(!re.is_empty());
        assert!(re.is_nullable());
    }

    #[test]
    fn test_derivative() {
        let re = Regex::symbol('a');
        let d = re.derivative('a');
        assert!(d.is_epsilon());

        let re = Regex::symbol('a');
        let d = re.derivative('b');
        assert!(d.is_empty());
    }

    #[test]
    fn test_nullable() {
        let re = Regex::epsilon();
        assert!(re.is_nullable());

        let re = Regex::symbol('a');
        assert!(!re.is_nullable());
    }

    #[test]
    fn test_complex_regex() {
        let re1 = Regex::symbol('a');
        let re2 = Regex::symbol('b');
        let re3 = Regex::symbol('c');
        let re4 = Regex::symbol('d');

        let complex_re = re1.concat(&re2).union(&re3.concat(&re4)).star();

        // Test if the complex regex is nullable
        assert!(complex_re.is_nullable());

        // Test derivatives
        let d1 = complex_re.derivative('a');
        assert!(!d1.is_empty());

        let d2 = d1.derivative('b');
        assert!(d2.is_nullable());

        let d3 = complex_re.derivative('c');
        assert!(!d3.is_empty());

        let d4 = d3.derivative('d');
        assert!(d4.is_nullable());

        let d5 = complex_re.derivative('e');
        assert!(d5.is_empty());
    }

    #[test]
    fn test_compl() {
        let re = Regex::symbol('a').compl();
        assert!(!re.is_empty());
        assert!(re.is_nullable());

        let re = Regex::epsilon().compl();
        assert!(!re.is_empty());
        assert!(!re.is_nullable());

        let re = Regex::empty().compl();
        assert!(!re.is_empty());
        assert!(re.is_nullable());
    }

    #[test]
    fn test_intersect() {
        let re1 = Regex::symbol('a');
        let re2 = Regex::symbol('a');
        let re = re1.intersect(&re2);
        assert!(!re.is_empty());
        assert!(!re.is_nullable());

        let re1 = Regex::symbol('a');
        let re2 = Regex::symbol('b');
        let re = re1.intersect(&re2);
        assert!(re.is_empty());
        assert!(!re.is_nullable());

        let re1 = Regex::range('a', 'z');
        let re2 = Regex::range('m', 'p');
        let re = re1.intersect(&re2);
        assert!(!re.is_empty());
        assert!(!re.is_nullable());
    }
}
