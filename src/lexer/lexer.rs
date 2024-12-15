use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::hash::Hash;

use crate::lexer::regex::{Matcher, Regex};

pub struct Lexer<T, M> {
    modes: Vec<Vec<Rule<T>>>,
    current_mode: usize,
    start_mode: usize,

    input: VecDeque<char>,
    position: usize,

    output: VecDeque<Lexeme<T>>,

    last_accepted: Option<(Lexeme<T>, usize)>,
    error: Option<String>,

    map: HashMap<M, usize>,
}

struct Rule<T> {
    token: T,
    //regex: Regex,
    matcher: Matcher,
    keep_span: bool,
    //from_mode: usize,
    to_mode: usize,
}

#[derive(Debug)]
pub struct Lexeme<T> {
    pub token: T,
    pub position: usize,
    pub length: usize,
    pub span: Option<String>,
}

impl<T, M> Lexer<T, M>
where
    T: Clone + Debug,
    M: Eq + Hash + Debug + Clone,
{
    pub fn new() -> Self {
        Self {
            modes: Vec::new(),
            current_mode: 0,
            start_mode: 0,

            input: VecDeque::new(),
            position: 0,

            output: VecDeque::new(),

            last_accepted: None,
            error: None,

            map: HashMap::new(),
        }
    }

    pub fn reset(&mut self) {
        for mode in self.modes.iter_mut() {
            for rule in mode.iter_mut() {
                rule.matcher.reset();
            }
        }

        self.input.clear();
        self.position = 0;
        self.output.clear();
        self.last_accepted = None;
        self.error = None;
        self.current_mode = self.start_mode;
    }

    pub fn set_start_mode(&mut self, mode: M) {
        self.current_mode = *self.map.entry(mode.clone()).or_insert_with(|| {
            let index = self.modes.len();
            self.modes.push(Vec::new());
            index
        });

        self.current_mode = self.map[&mode];
    }

    pub fn add_rule(&mut self, regex: &Regex, token: T, keep_span: bool, from_mode: M, to_mode: M) {
        let from_mode_index = *self.map.entry(from_mode.clone()).or_insert_with(|| {
            let index = self.modes.len();
            self.modes.push(Vec::new());

            index
        });

        let to_mode_index = *self.map.entry(to_mode).or_insert_with(|| {
            let index = self.modes.len();
            self.modes.push(Vec::new());
            index
        });

        let matcher = Matcher::new(regex);

        self.modes[from_mode_index].push(Rule {
            token,
            //regex: regex.clone(),
            matcher,
            keep_span,
            //from_mode: from_mode_index,
            to_mode: to_mode_index,
        });
    }

    pub fn put(&mut self, c: char) {
        if self.error.is_some() {
            return;
        }

        self.input.push_back(c);

        self.lex();
    }

    pub fn finish(&mut self) {
        if self.error.is_some() {
            return;
        }

        self.emit();

        if self.input.len() > 0 {
            self.error = Some("unexpected end of input".to_string());
        }
    }

    pub fn get(&mut self) -> Option<Lexeme<T>> {
        if self.error.is_some() {
            return None;
        }

        if let Some(lexeme) = self.output.pop_front() {
            Some(lexeme)
        } else {
            None
        }
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    fn lex(&mut self) {
        // assume this is called immediately after a new character is put into the input.
        let mut all_dead = true;

        let mut cursor = self.input.len() - 1;
        let mut accepted = None;

        while cursor < self.input.len() {
            let c = self.input[cursor];

            for rule in self.modes[self.current_mode].iter_mut() {
                rule.matcher.put(c);

                if rule.matcher.is_accepted() {
                    if accepted.is_none() {
                        let lexeme = Lexeme {
                            token: rule.token.clone(),
                            position: self.position,
                            length: cursor + 1,
                            span: if rule.keep_span {
                                Some(self.input.iter().take(cursor + 1).collect())
                            } else {
                                None
                            },
                        };

                        accepted = Some((lexeme, rule.to_mode));
                    }
                }

                all_dead &= rule.matcher.is_dead();
            }

            if accepted.is_some() {
                self.last_accepted = accepted.take();
            }

            if all_dead {
                self.emit();

                if self.error.is_some() {
                    break;
                }

                cursor = 0;
            } else {
                cursor += 1;
            }
        }
    }

    fn emit(&mut self) {
        if let Some((lexeme, mode_to)) = self.last_accepted.take() {
            self.position += lexeme.length;
            self.input.drain(..lexeme.length);
            self.current_mode = mode_to;

            for rule in self.modes[self.current_mode].iter_mut() {
                rule.matcher.reset();
            }

            self.output.push_back(lexeme);
        } else {
            self.error = Some("unexpected character".to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::regex::Regex;

    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    enum Token {
        A,
        B,
    }

    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    enum Mode {
        Initial,
        Secondary,
    }

    #[test]
    fn test_lexer_single_token() {
        let mut lexer = Lexer::new();
        lexer.add_rule(
            &Regex::char('a'),
            Token::A,
            false,
            Mode::Initial,
            Mode::Initial,
        );

        lexer.put('a');
        lexer.finish();
        let lexeme = lexer.get().unwrap();

        assert_eq!(lexeme.token, Token::A);
        assert_eq!(lexeme.position, 0);
        assert_eq!(lexeme.length, 1);
        assert!(lexer.get().is_none());
    }

    #[test]
    fn test_lexer_multiple_tokens() {
        let mut lexer = Lexer::new();
        lexer.add_rule(
            &Regex::char('a'),
            Token::A,
            false,
            Mode::Initial,
            Mode::Initial,
        );
        lexer.add_rule(
            &Regex::char('b'),
            Token::B,
            false,
            Mode::Initial,
            Mode::Initial,
        );

        lexer.put('a');
        lexer.put('b');
        lexer.finish();

        let lexeme_a = lexer.get().unwrap();
        assert_eq!(lexeme_a.token, Token::A);
        assert_eq!(lexeme_a.position, 0);
        assert_eq!(lexeme_a.length, 1);
        assert!(lexer.error.is_none());
        let lexeme_b = lexer.get().unwrap();
        assert_eq!(lexeme_b.token, Token::B);
        assert_eq!(lexeme_b.position, 1);
        assert_eq!(lexeme_b.length, 1);

        assert!(lexer.get().is_none());
    }

    #[test]
    fn test_lexer_mode_switching() {
        let mut lexer = Lexer::new();
        lexer.add_rule(
            &Regex::char('a'),
            Token::A,
            true,
            Mode::Initial,
            Mode::Secondary,
        );
        lexer.add_rule(
            &Regex::char('b'),
            Token::B,
            true,
            Mode::Secondary,
            Mode::Initial,
        );
        lexer.set_start_mode(Mode::Initial);

        lexer.put('a');
        lexer.put('b');

        let lexeme_a = lexer.get().unwrap();
        assert_eq!(lexeme_a.token, Token::A);
        assert_eq!(lexeme_a.position, 0);
        assert_eq!(lexeme_a.length, 1);

        lexer.finish();

        let lexeme_b = lexer.get().unwrap();
        assert_eq!(lexeme_b.token, Token::B);
        assert_eq!(lexeme_b.position, 1);
        assert_eq!(lexeme_b.length, 1);

        assert!(lexer.get().is_none());
    }

    #[test]
    fn test_lexer_error_handling() {
        let mut lexer = Lexer::new();
        lexer.add_rule(
            &Regex::char('a'),
            Token::A,
            false,
            Mode::Initial,
            Mode::Initial,
        );

        lexer.put('b');
        lexer.finish();
        assert!(lexer.get().is_none());
        assert_eq!(lexer.error(), Some("unexpected character"));
    }

    #[test]
    fn test_lexer_complex_token() {
        use crate::lexer::token::Token;
        let mut lexer = Lexer::new();
        lexer.add_rule(
            &Token::Identifier.regex(),
            Token::Identifier,
            false,
            Mode::Initial,
            Mode::Initial,
        );

        let ident = "abc23245213";
        for c in ident.chars() {
            lexer.put(c);
        }

        lexer.finish();

        let lexeme = lexer.get().unwrap();

        assert_eq!(lexeme.token, Token::Identifier);
        assert_eq!(lexeme.position, 0);
        assert_eq!(lexeme.length, ident.len());
        assert!(lexer.get().is_none());

        lexer.reset();

        let not_ident = "12345";
        for c in not_ident.chars() {
            lexer.put(c);
        }

        lexer.finish();

        assert!(lexer.error().is_some());
        assert!(lexer.get().is_none());
    }

    #[test]
    fn test_lexer_two_complex_token() {
        use crate::lexer::token::Token;

        let mut lexer = Lexer::new();
        lexer.add_rule(
            &Token::Identifier.regex(),
            Token::Identifier,
            false,
            Mode::Initial,
            Mode::Initial,
        );

        lexer.add_rule(
            &Token::Integer.regex(),
            Token::Integer,
            false,
            Mode::Initial,
            Mode::Initial,
        );

        lexer.add_rule(
            &Token::Whitespace.regex(),
            Token::Whitespace,
            false,
            Mode::Initial,
            Mode::Initial,
        );

        let stream = "abc 123";
        for c in stream.chars() {
            lexer.put(c);
        }

        lexer.finish();

        assert!(lexer.error().is_none());
        assert!(lexer.get().unwrap().token == Token::Identifier);
        assert!(lexer.get().unwrap().token == Token::Whitespace);
        assert!(lexer.get().unwrap().token == Token::Integer);
        assert!(lexer.get().is_none());
    }
}
