use std::fmt::Debug;
use std::rc::Rc;

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
    Star(Regex),
}

impl Debug for Regex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &*self.node {
            RegexNode::Empty => write!(f, "∅"),
            RegexNode::Epsilon => write!(f, "ε"),
            RegexNode::Range(lo, hi) => write!(f, "(range {:?} {:?})", lo, hi),
            RegexNode::Concat(a, b) => write!(f, "(concat {:?} {:?})", a, b),
            RegexNode::Union(a, b) => write!(f, "(union {:?} {:?})", a, b),
            RegexNode::Star(a) => write!(f, "(star {:?})", a),
        }
    }
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
        if self.is_empty() || other.is_epsilon() {
            return self.clone();
        }

        if other.is_empty() || self.is_epsilon() {
            return other.clone();
        }

        Self {
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

        Self {
            node: Rc::new(RegexNode::Union(self.clone(), other.clone())),
            nullable: self.nullable || other.nullable,
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

    pub fn one_of(chars: &str) -> Self {
        let mut r = Self::empty();

        for c in chars.chars() {
            r = r.union(&Self::char(c));
        }

        r
    }

    pub fn none_of(chars: &str) -> Self {
        let mut r = Self::empty();

        let mut chars = chars.chars().collect::<Vec<_>>();

        chars.sort_unstable();
        chars.dedup();

        for (&a, &b) in chars.iter().zip(chars.iter().skip(1)) {
            let anext = incr_char(a);
            let bprev = decr_char(b);

            if bprev != a {
                r = r.union(&Self::range(anext, bprev));
            }
        }

        if let Some(a) = chars.first() {
            if *a != char::MIN {
                r = r.union(&Self::range(char::MIN, decr_char(*a)));
            }
        }

        if let Some(b) = chars.last() {
            if *b != char::MAX {
                r = r.union(&Self::range(incr_char(*b), char::MAX));
            }
        }

        r
    }

    fn is_nullable(&self) -> bool {
        self.nullable
    }

    fn v(&self) -> Self {
        if self.is_nullable() {
            Self::epsilon()
        } else {
            Self::empty()
        }
    }

    fn derivative(&self, c: char) -> Self {
        match &*self.node {
            RegexNode::Empty => Self::empty(),
            RegexNode::Epsilon => Self::empty(),
            RegexNode::Range(lo, hi) => {
                if *lo <= c && c <= *hi {
                    Self::epsilon()
                } else {
                    Self::empty()
                }
            }
            RegexNode::Concat(a, b) => {
                let da = a.derivative(c);
                let db = b.derivative(c);
                let va = a.v();

                da.concat(b).union(&va.concat(&db))
            }
            RegexNode::Union(a, b) => a.derivative(c).union(&b.derivative(c)),
            RegexNode::Star(a) => a.derivative(c).concat(self),
        }
    }
}

fn incr_char(c: char) -> char {
    assert!(c != char::MAX, "cannot increment MAX char");

    if c as u32 == 0xD7FF {
        char::from_u32(0xE000).unwrap()
    } else {
        char::from_u32(c as u32 + 1).unwrap()
    }
}

fn decr_char(c: char) -> char {
    assert!(c != char::MIN, "cannot decrement MIN char");

    if c as u32 == 0xE000 {
        char::from_u32(0xD7FF).unwrap()
    } else {
        char::from_u32(c as u32 - 1).unwrap()
    }
}

pub struct Matcher {
    regex: Regex,
    state: Regex,
}

impl Matcher {
    pub fn new(regex: &Regex) -> Self {
        Self {
            regex: regex.clone(),
            state: regex.clone(),
        }
    }

    pub fn reset(&mut self) {
        self.state = self.regex.clone();
    }

    pub fn put(&mut self, c: char) {
        self.state = self.state.derivative(c);
    }

    pub fn is_accepted(&self) -> bool {
        self.state.is_nullable()
    }

    pub fn is_dead(&self) -> bool {
        self.state.is_empty()
    }

    pub fn match_string(&mut self, s: &str) -> bool {
        self.reset();

        for c in s.chars() {
            self.put(c);
        }

        self.is_accepted()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_matcher_empty() {
        let regex = Regex::empty();
        let mut matcher = Matcher::new(&regex);

        assert!(!matcher.is_accepted());
        matcher.put('a');
        assert!(!matcher.is_accepted());
        assert!(matcher.is_dead());
    }

    #[test]
    fn test_matcher_epsilon() {
        let regex = Regex::epsilon();
        let mut matcher = Matcher::new(&regex);

        assert!(matcher.is_accepted());
        matcher.put('a');
        assert!(!matcher.is_accepted());
        assert!(matcher.is_dead());
    }

    #[test]
    fn test_matcher_char() {
        let regex = Regex::char('a');
        let mut matcher = Matcher::new(&regex);

        assert!(!matcher.is_accepted());
        matcher.put('a');
        assert!(matcher.is_accepted());
        matcher.put('b');
        assert!(!matcher.is_accepted());
        assert!(matcher.is_dead());
    }

    #[test]
    fn test_matcher_concat() {
        let regex = Regex::char('a').concat(&Regex::char('b'));
        let mut matcher = Matcher::new(&regex);

        assert!(!matcher.is_accepted());
        matcher.put('a');
        assert!(!matcher.is_accepted());
        matcher.put('b');
        assert!(matcher.is_accepted());
        matcher.put('c');
        assert!(!matcher.is_accepted());
        assert!(matcher.is_dead());
    }

    #[test]
    fn test_matcher_union() {
        let regex = Regex::char('a').union(&Regex::char('b'));
        let mut matcher = Matcher::new(&regex);

        assert!(!matcher.is_accepted());
        matcher.put('a');
        assert!(matcher.is_accepted());
        matcher.reset();
        matcher.put('b');
        assert!(matcher.is_accepted());
        matcher.reset();
        matcher.put('c');
        assert!(!matcher.is_accepted());
        assert!(matcher.is_dead());
    }

    #[test]
    fn test_matcher_star() {
        let regex = Regex::char('a').star();
        let mut matcher = Matcher::new(&regex);

        assert!(matcher.is_accepted());
        matcher.put('a');
        assert!(matcher.is_accepted());
        matcher.put('a');
        assert!(matcher.is_accepted());
        matcher.put('b');
        assert!(!matcher.is_accepted());
        assert!(matcher.is_dead());
    }

    #[test]
    fn test_matcher_plus() {
        let regex = Regex::char('a').plus();
        let mut matcher = Matcher::new(&regex);

        assert!(!matcher.is_accepted());
        matcher.put('a');
        assert!(matcher.is_accepted());
        matcher.put('a');
        assert!(matcher.is_accepted());
        matcher.put('b');
        assert!(!matcher.is_accepted());
        assert!(matcher.is_dead());
    }

    #[test]
    fn test_matcher_opt() {
        let regex = Regex::char('a').opt();
        let mut matcher = Matcher::new(&regex);

        assert!(matcher.is_accepted());
        matcher.put('a');
        assert!(matcher.is_accepted());
        matcher.put('b');
        assert!(!matcher.is_accepted());
        assert!(matcher.is_dead());
    }

    #[test]
    fn test_matcher_one_of() {
        let regex = Regex::one_of("abc");
        let mut matcher = Matcher::new(&regex);

        assert!(!matcher.is_accepted());
        matcher.put('a');
        assert!(matcher.is_accepted());
        matcher.reset();
        matcher.put('b');
        assert!(matcher.is_accepted());
        matcher.reset();
        matcher.put('c');
        assert!(matcher.is_accepted());
        matcher.reset();
        matcher.put('d');
        assert!(!matcher.is_accepted());
        assert!(matcher.is_dead());
    }

    #[test]
    fn test_matcher_none_of() {
        let regex = Regex::none_of("abc");
        let mut matcher = Matcher::new(&regex);

        println!("{:?}", regex);

        assert!(!matcher.is_accepted());
        matcher.put('a');
        assert!(!matcher.is_accepted());
        assert!(matcher.is_dead());
        matcher.reset();
        matcher.put('d');
        assert!(matcher.is_accepted());
    }
}
