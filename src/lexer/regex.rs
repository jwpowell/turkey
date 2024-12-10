use super::nfa::Nfa;

pub struct Regex;

pub fn empty() -> Regex {
    Regex::empty()
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
    pub fn empty() -> Self {
        todo!()
    }

    pub fn any() -> Self {
        todo!()
    }

    pub fn char(c: char) -> Self {
        todo!()
    }

    pub fn range(lo: char, hi: char) -> Self {
        todo!()
    }

    pub fn concat(&self, other: &Self) -> Self {
        todo!()
    }

    pub fn union(&self, other: &Self) -> Self {
        todo!()
    }

    pub fn intersect(&self, other: &Self) -> Self {
        todo!()
    }

    pub fn star(&self) -> Self {
        todo!()
    }

    pub fn plus(&self) -> Self {
        todo!()
    }

    pub fn opt(&self) -> Self {
        todo!()
    }

    pub fn not(&self) -> Self {
        todo!()
    }

    pub fn one_of(chars: &str) -> Self {
        todo!()
    }

    pub fn none_of(chars: &str) -> Self {
        todo!()
    }

    pub fn nfa<T: Clone>(&self, output: T) -> Nfa<T> {
        todo!()
    }
}
