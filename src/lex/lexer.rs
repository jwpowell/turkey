use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::hash::Hash;

use crate::lex::nfa::Nfa;
use crate::lex::regex::Regex;

pub struct Lexer<M, T> {
    modes: Vec<Vec<Rule<T>>>,
    mode_indices: HashMap<M, usize>,
    mode_names: HashMap<usize, M>,

    start_mode: usize,
    current_mode: usize,

    input: VecDeque<char>,
    cursor: usize,
    position: usize,
    last_accepted: Option<(usize, usize)>,

    output: VecDeque<Lexeme<T>>,
    error: Option<LexerError>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Lexeme<T> {
    pub token: T,
    pub position: usize,
    pub length: usize,
    pub span: Option<String>,
}

pub struct LexerError {
    pub message: String,
    pub position: usize,
}

pub struct Rule<T> {
    token: T,
    nfa: Nfa,
    mode_to: usize,
    keep_span: bool,
}

impl<M, T> Lexer<M, T>
where
    T: Clone + Debug,
    M: Copy + Debug + Eq + Hash + Default,
{
    pub fn new() -> Self {
        Lexer {
            modes: vec![],
            mode_indices: HashMap::new(),
            mode_names: HashMap::new(),
            start_mode: 0,
            current_mode: 0,
            input: VecDeque::new(),
            cursor: 0,
            position: 0,
            last_accepted: None,
            output: VecDeque::new(),
            error: None,
        }
    }

    fn get_mode_index(&mut self, mode: M) -> usize {
        *self.mode_indices.entry(mode).or_insert_with(|| {
            let index = self.modes.len();
            self.modes.push(vec![]);
            self.mode_names.insert(index, mode);
            index
        })
    }

    pub fn set_start_mode(&mut self, mode: M) {
        self.start_mode = self.get_mode_index(mode);
    }

    pub fn add_rule(&mut self, token: T, regex: &Regex, mode_from: M, mode_to: M, keep_span: bool) {
        let mode_from = self.get_mode_index(mode_from);
        let mode_to = self.get_mode_index(mode_to);
        let nfa = regex.to_nfa();
        self.modes[mode_from].push(Rule {
            token,
            nfa,
            mode_to,
            keep_span,
        });
    }

    pub fn reset(&mut self) {
        self.current_mode = self.start_mode;
        self.cursor = 0;
        self.position = 0;
        self.input.clear();
        self.last_accepted = None;
        self.output.clear();
        self.error = None;
    }

    pub fn put(&mut self, c: char) {
        if self.is_error() {
            return;
        }

        self.input.push_back(c);
        self.lex();
    }

    pub fn finish(&mut self) {
        if self.is_error() {
            return;
        }

        self.emit();

        if self.cursor < self.input.len() {
            self.error = Some(LexerError {
                message: "unexpected end of input".to_string(),
                position: self.cursor,
            });
        }
    }

    pub fn get(&mut self) -> Option<Lexeme<T>> {
        self.output.pop_front()
    }

    pub fn is_error(&self) -> bool {
        self.get_error().is_some()
    }

    pub fn get_error(&self) -> Option<&LexerError> {
        self.error.as_ref()
    }

    fn lex(&mut self) {
        let mut all_dead = true;
        let mut last_accepted = None;
        while self.cursor < self.input.len() {
            let c = self.input[self.cursor];

            for (i, rule) in self.modes[self.current_mode].iter_mut().enumerate() {
                rule.nfa.put(c);
                all_dead &= rule.nfa.is_dead();

                if rule.nfa.is_accept() {
                    last_accepted = Some((i, self.cursor + 1));
                }
            }

            if all_dead {
                self.emit();
                continue;
            }

            if last_accepted.is_some() {
                self.last_accepted = last_accepted;
            }

            self.cursor += 1;
        }
    }

    fn emit(&mut self) {
        if self.last_accepted.is_none() {
            self.error = Some(LexerError {
                message: "unexpected input".to_string(),
                position: self.cursor,
            });
        }

        let (rule, length) = self.last_accepted.unwrap();

        let token = self.modes[self.current_mode][rule].token.clone();
        let position = self.position;
        let span = if self.modes[self.current_mode][rule].keep_span {
            Some(
                self.input
                    .iter()
                    .take(length)
                    .collect::<String>()
                    .to_string(),
            )
        } else {
            None
        };

        self.output.push_back(Lexeme {
            token,
            position,
            length,
            span,
        });

        self.position += length;
        self.current_mode = self.modes[self.current_mode][rule].mode_to;
        self.cursor = 0;
        self.last_accepted = None;
        self.input.drain(..length);

        for rule in self.modes[self.current_mode].iter_mut() {
            rule.nfa.reset();
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    enum Token {
        LParen,
        RParen,
        Semicolon,
        Whitespace,
        Newline,
        Comment,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
    enum Mode {
        #[default]
        Default,
        Comment,
    }

    fn small_lexer() -> Lexer<Mode, Token> {
        let lparen = Regex::char('(');
        let rparen = Regex::char(')');
        let semicolon = Regex::char(';');
        let whitespace = Regex::one_of(" \t").star();
        let newline = Regex::char('\n');
        let comment = Regex::none_of("\n").star();

        let mut lexer = Lexer::new();

        lexer.set_start_mode(Mode::Default);
        lexer.add_rule(Token::LParen, &lparen, Mode::Default, Mode::Default, false);
        lexer.add_rule(Token::RParen, &rparen, Mode::Default, Mode::Default, false);
        lexer.add_rule(
            Token::Semicolon,
            &semicolon,
            Mode::Default,
            Mode::Comment,
            false,
        );
        lexer.add_rule(
            Token::Whitespace,
            &whitespace,
            Mode::Default,
            Mode::Default,
            false,
        );
        lexer.add_rule(
            Token::Newline,
            &newline,
            Mode::Default,
            Mode::Default,
            false,
        );

        lexer.add_rule(Token::Comment, &comment, Mode::Comment, Mode::Default, true);

        lexer
    }

    fn test_lexer<M, T>(lexer: &mut Lexer<M, T>, input: &str, expected: &[Lexeme<T>])
    where
        T: Clone + Debug + Eq,
        M: Copy + Debug + Eq + Hash + Default,
    {
        lexer.reset();
        for c in input.chars() {
            lexer.put(c);
        }
        lexer.finish();

        let mut actual = Vec::new();
        while let Some(lexeme) = lexer.get() {
            actual.push(lexeme);
        }

        for (a, e) in actual.iter().zip(expected.iter()) {
            println!("{:?}\n{:?}\n", a, e);
            assert_eq!(a, e);
        }

        assert!(lexer.get_error().is_none());
        assert!(!lexer.is_error());
    }

    #[test]
    fn test_simple_tokens() {
        let mut lexer = small_lexer();
        test_lexer(
            &mut lexer,
            "()",
            &[
                Lexeme {
                    token: Token::LParen,
                    position: 0,
                    length: 1,
                    span: None,
                },
                Lexeme {
                    token: Token::RParen,
                    position: 1,
                    length: 1,
                    span: None,
                },
            ],
        );
    }

    #[test]
    fn test_whitespace() {
        let mut lexer = small_lexer();
        test_lexer(
            &mut lexer,
            "  (  )  ",
            &[
                Lexeme {
                    token: Token::Whitespace,
                    position: 0,
                    length: 2,
                    span: None,
                },
                Lexeme {
                    token: Token::LParen,
                    position: 2,
                    length: 1,
                    span: None,
                },
                Lexeme {
                    token: Token::Whitespace,
                    position: 3,
                    length: 2,
                    span: None,
                },
                Lexeme {
                    token: Token::RParen,
                    position: 5,
                    length: 1,
                    span: None,
                },
                Lexeme {
                    token: Token::Whitespace,
                    position: 6,
                    length: 2,
                    span: None,
                },
            ],
        );
    }

    #[test]
    fn test_comment() {
        let mut lexer = small_lexer();
        test_lexer(
            &mut lexer,
            "(; this is a comment\n)",
            &[
                Lexeme {
                    token: Token::LParen,
                    position: 0,
                    length: 1,
                    span: None,
                },
                Lexeme {
                    token: Token::Semicolon,
                    position: 1,
                    length: 1,
                    span: None,
                },
                Lexeme {
                    token: Token::Comment,
                    position: 2,
                    length: 18,
                    span: Some(" this is a comment".to_string()),
                },
                Lexeme {
                    token: Token::Newline,
                    position: 20,
                    length: 1,
                    span: None,
                },
                Lexeme {
                    token: Token::RParen,
                    position: 21,
                    length: 1,
                    span: None,
                },
            ],
        );
    }
}
