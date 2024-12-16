use std::fmt::Debug;
use std::rc::Rc;

use crate::lex::nfa::Nfa;

#[derive(Debug, Clone)]
pub struct Regex(Rc<RegexInner>);

#[derive(Debug)]
enum RegexInner {
    Empty,
    Epsilon,
    Char(char),
    Range(char, char),
    OneOf(String),
    NoneOf(String),
    Any,
    Concat(Regex, Regex),
    Union(Regex, Regex),
    Star(Regex),
    Plus(Regex),
    Optional(Regex),
}

impl Regex {
    pub fn empty() -> Self {
        Regex(Rc::new(RegexInner::Empty))
    }

    pub fn epsilon() -> Self {
        Regex(Rc::new(RegexInner::Epsilon))
    }

    pub fn char(c: char) -> Self {
        Regex(Rc::new(RegexInner::Char(c)))
    }

    pub fn range(start: char, end: char) -> Self {
        Regex(Rc::new(RegexInner::Range(start, end)))
    }

    pub fn one_of(chars: &str) -> Self {
        Regex(Rc::new(RegexInner::OneOf(chars.to_string())))
    }

    pub fn none_of(chars: &str) -> Self {
        Regex(Rc::new(RegexInner::NoneOf(chars.to_string())))
    }

    pub fn any() -> Self {
        Regex(Rc::new(RegexInner::Any))
    }

    pub fn concat(&self, other: &Self) -> Self {
        Regex(Rc::new(RegexInner::Concat(self.clone(), other.clone())))
    }

    pub fn union(&self, other: &Self) -> Self {
        Regex(Rc::new(RegexInner::Union(self.clone(), other.clone())))
    }

    pub fn star(&self) -> Self {
        Regex(Rc::new(RegexInner::Star(self.clone())))
    }

    pub fn plus(&self) -> Self {
        Regex(Rc::new(RegexInner::Plus(self.clone())))
    }

    pub fn optional(&self) -> Self {
        Regex(Rc::new(RegexInner::Optional(self.clone())))
    }

    pub fn to_nfa(&self) -> Nfa {
        let mut nfa = match &*self.0 {
            RegexInner::Empty => to_nfa_empty(),
            RegexInner::Epsilon => to_nfa_epsilon(),
            RegexInner::Char(c) => to_nfa_char(*c),
            RegexInner::Range(lo, hi) => to_nfa_range(*lo, *hi),
            RegexInner::OneOf(chars) => to_nfa_one_of(chars),
            RegexInner::NoneOf(chars) => to_nfa_none_of(chars),
            RegexInner::Any => to_nfa_any(),
            RegexInner::Concat(lhs, rhs) => to_nfa_concat(&lhs.to_nfa(), &rhs.to_nfa()),
            RegexInner::Union(lhs, rhs) => to_nfa_union(&lhs.to_nfa(), &rhs.to_nfa()),
            RegexInner::Star(regex) => to_nfa_star(&regex.to_nfa()),
            RegexInner::Plus(regex) => to_nfa_plus(&regex.to_nfa()),
            RegexInner::Optional(regex) => to_nfa_optional(&regex.to_nfa()),
        };

        nfa.reset();

        nfa
    }
}

fn to_nfa_empty() -> Nfa {
    Nfa::new()
}

fn to_nfa_epsilon() -> Nfa {
    let mut nfa = Nfa::new();

    let start = nfa.create_node();
    let accept = nfa.create_node();

    nfa.add_start(start);
    nfa.add_accept(accept);
    nfa.add_epsilon(start, accept);

    nfa
}

fn to_nfa_char(c: char) -> Nfa {
    to_nfa_range(c, c)
}

fn to_nfa_range(lo: char, hi: char) -> Nfa {
    let mut nfa = Nfa::new();

    let start = nfa.create_node();
    let accept = nfa.create_node();

    nfa.add_start(start);
    nfa.add_accept(accept);
    nfa.add_edge(start, lo, hi, accept);

    nfa
}

fn to_nfa_one_of(chars: &str) -> Nfa {
    if chars.is_empty() {
        return to_nfa_empty();
    }

    let mut nfa = Nfa::new();

    let start = nfa.create_node();
    let accept = nfa.create_node();

    nfa.add_start(start);
    nfa.add_accept(accept);

    for c in chars.chars() {
        nfa.add_edge(start, c, c, accept);
    }

    nfa
}

fn char_decr(c: char) -> char {
    assert!(c != char::MIN);

    if c == '\u{E000}' {
        return '\u{D7FF}';
    }

    char::from_u32(c as u32 - 1).unwrap()
}

fn char_incr(c: char) -> char {
    assert!(c != char::MAX);

    if c == '\u{D7FF}' {
        return '\u{E000}';
    }

    char::from_u32(c as u32 + 1).unwrap()
}

fn to_nfa_none_of(chars: &str) -> Nfa {
    if chars.is_empty() {
        return to_nfa_any();
    }

    let mut nfa = Nfa::new();

    let start = nfa.create_node();
    let accept = nfa.create_node();

    nfa.add_start(start);
    nfa.add_accept(accept);

    let inner_ranges = chars.chars().zip(chars.chars().skip(1));

    let mut sub_nfas = Vec::new();
    for (a, b) in inner_ranges {
        if char_incr(a) <= char_decr(b) {
            let nfa_range = to_nfa_range(char_incr(a), char_decr(b));
            sub_nfas.push(nfa_range);
        }
    }

    let first = chars.chars().next().unwrap();
    let last = chars.chars().last().unwrap();

    if first != char::MIN {
        let nfa_range = to_nfa_range(char::MIN, char_decr(first));
        sub_nfas.push(nfa_range);
    }

    if last != char::MAX {
        let nfa_range = to_nfa_range(char_incr(last), char::MAX);
        sub_nfas.push(nfa_range);
    }

    sub_nfas
        .into_iter()
        .reduce(|a, b| to_nfa_union(&a, &b))
        .unwrap()
}

fn to_nfa_any() -> Nfa {
    to_nfa_range(char::MIN, char::MAX)
}

fn to_nfa_concat(lhs: &Nfa, rhs: &Nfa) -> Nfa {
    let mut nfa = Nfa::new();

    let start = nfa.create_node();
    let accept = nfa.create_node();

    nfa.add_start(start);
    nfa.add_accept(accept);

    let (lhs_start, lhs_accept) = nfa.merge(&lhs);
    let (rhs_start, rhs_accept) = nfa.merge(&rhs);

    nfa.add_epsilon(start, lhs_start);
    nfa.add_epsilon(lhs_accept, rhs_start);
    nfa.add_epsilon(rhs_accept, accept);

    nfa
}

fn to_nfa_union(lhs: &Nfa, rhs: &Nfa) -> Nfa {
    let mut nfa = Nfa::new();

    let start = nfa.create_node();
    let accept = nfa.create_node();

    nfa.add_start(start);
    nfa.add_accept(accept);

    let (lhs_start, lhs_accept) = nfa.merge(&lhs);
    let (rhs_start, rhs_accept) = nfa.merge(&rhs);

    nfa.add_epsilon(start, lhs_start);
    nfa.add_epsilon(start, rhs_start);
    nfa.add_epsilon(lhs_accept, accept);
    nfa.add_epsilon(rhs_accept, accept);

    nfa
}

fn to_nfa_star(nfa: &Nfa) -> Nfa {
    let mut nfa_star = Nfa::new();

    let start = nfa_star.create_node();
    let accept = nfa_star.create_node();

    nfa_star.add_start(start);
    nfa_star.add_accept(accept);

    let (inner_start, inner_accept) = nfa_star.merge(&nfa);

    nfa_star.add_epsilon(start, accept);
    nfa_star.add_epsilon(start, inner_start);
    nfa_star.add_epsilon(inner_accept, accept);
    nfa_star.add_epsilon(accept, start);

    nfa_star
}

fn to_nfa_plus(nfa: &Nfa) -> Nfa {
    to_nfa_concat(&nfa, &to_nfa_star(nfa))
}

fn to_nfa_optional(nfa: &Nfa) -> Nfa {
    to_nfa_union(&to_nfa_epsilon(), &nfa)
}

#[cfg(test)]
mod test {
    use super::*;

    fn test_regex(regex: &Regex, s: &str, expected: bool) {
        let mut nfa = regex.to_nfa();

        for c in s.chars() {
            nfa.put(c);
        }

        assert_eq!(nfa.is_accept(), expected);
    }

    #[test]
    fn test_char() {
        let regex = Regex::char('a');
        test_regex(&regex, "a", true);
        test_regex(&regex, "b", false);
        test_regex(&regex, "", false);
        test_regex(&regex, "aa", false);
    }

    #[test]
    fn test_concat() {
        let regex = Regex::char('a').concat(&Regex::char('b'));
        test_regex(&regex, "ab", true);
        test_regex(&regex, "a", false);
        test_regex(&regex, "b", false);
        test_regex(&regex, "ba", false);
        test_regex(&regex, "", false);
    }

    #[test]
    fn test_union() {
        let regex = Regex::char('a').union(&Regex::char('b'));
        test_regex(&regex, "a", true);
        test_regex(&regex, "b", true);
        test_regex(&regex, "ab", false);
        test_regex(&regex, "", false);
    }

    #[test]
    fn test_star() {
        let regex = Regex::char('a').star();
        test_regex(&regex, "", true);
        test_regex(&regex, "a", true);
        test_regex(&regex, "aa", true);
        test_regex(&regex, "aaa", true);
        test_regex(&regex, "b", false);
        test_regex(&regex, "ab", false);
    }

    #[test]
    fn test_plus() {
        let regex = Regex::char('a').plus();
        test_regex(&regex, "", false);
        test_regex(&regex, "a", true);
        test_regex(&regex, "aa", true);
        test_regex(&regex, "aaa", true);
        test_regex(&regex, "b", false);
        test_regex(&regex, "ab", false);
    }

    #[test]
    fn test_optional() {
        let regex = Regex::char('a').optional();
        test_regex(&regex, "", true);
        test_regex(&regex, "a", true);
        test_regex(&regex, "aa", false);
        test_regex(&regex, "b", false);
    }

    #[test]
    fn test_complex() {
        let regex = Regex::char('a')
            .concat(&Regex::char('b').star())
            .concat(&Regex::char('c'));
        test_regex(&regex, "ac", true);
        test_regex(&regex, "abc", true);
        test_regex(&regex, "abbc", true);
        test_regex(&regex, "abbbc", true);
        test_regex(&regex, "a", false);
        test_regex(&regex, "ab", false);
        test_regex(&regex, "bc", false);
        test_regex(&regex, "", false);
    }
}
