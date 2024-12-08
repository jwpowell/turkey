use crate::regex::Regex;

pub struct Matcher {
    regex: Regex,
    state: Regex,
}

impl Matcher {
    pub fn from_regex(r: &Regex) -> Self {
        Matcher {
            regex: r.clone(),
            state: r.clone(),
        }
    }

    pub fn reset(&mut self) {
        self.state = self.regex.clone();
    }

    pub fn put(&mut self, c: char) {
        self.state = self.state.derivative(c);
    }

    pub fn is_accepting(&self) -> bool {
        self.state.is_nullable()
    }

    pub fn is_dead(&self) -> bool {
        self.state.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matcher_from_regex() {
        let re = Regex::symbol('a');
        let matcher = Matcher::from_regex(&re);
        assert!(!matcher.is_accepting());
        assert!(!matcher.is_dead());
    }

    #[test]
    fn test_matcher_reset() {
        let re = Regex::symbol('a');
        let mut matcher = Matcher::from_regex(&re);
        matcher.put('a');
        assert!(matcher.is_accepting());
        matcher.reset();
        assert!(!matcher.is_accepting());
    }

    #[test]
    fn test_matcher_put() {
        let re = Regex::symbol('a');
        let mut matcher = Matcher::from_regex(&re);
        matcher.put('a');
        assert!(matcher.is_accepting());
        matcher.put('b');
        assert!(matcher.is_dead());
    }

    #[test]
    fn test_matcher_is_accepting() {
        let re = Regex::symbol('a');
        let mut matcher = Matcher::from_regex(&re);
        assert!(!matcher.is_accepting());
        matcher.put('a');
        assert!(matcher.is_accepting());
    }

    #[test]
    fn test_matcher_is_dead() {
        let re = Regex::symbol('a');
        let mut matcher = Matcher::from_regex(&re);
        assert!(!matcher.is_dead());
        matcher.put('b');
        assert!(matcher.is_dead());
    }

    #[test]
    fn test_matcher_complex_regex() {
        let re1 = Regex::symbol('a');
        let re2 = Regex::symbol('b');
        let re3 = Regex::symbol('c');
        let re4 = Regex::symbol('d');

        let complex_re = re1.concat(&re2).union(&re3.concat(&re4)).star();
        let mut matcher = Matcher::from_regex(&complex_re);

        // Test if the matcher is not accepting initially
        assert!(matcher.is_accepting());

        // Test matcher with a sequence that should be accepted
        matcher.put('a');
        assert!(!matcher.is_dead());
        matcher.put('b');
        assert!(matcher.is_accepting());

        matcher.reset();

        matcher.put('c');
        assert!(!matcher.is_dead());
        matcher.put('d');
        assert!(matcher.is_accepting());

        matcher.reset();

        // Test matcher with a sequence that should not be accepted
        matcher.put('e');
        assert!(matcher.is_dead());
    }
}
