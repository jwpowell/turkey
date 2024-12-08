use std::collections::VecDeque;

use crate::matcher::Matcher;
use crate::regex::Regex;

pub trait Token: Sized {
    type Kind: Sized + Copy;

    fn create(kind: Self::Kind, position: usize, span: &String) -> Self;
}

pub struct Lexer<T: Token, M> {
    span: VecDeque<char>,
    cursor: usize,

    position: usize,

    pending: Option<(usize, usize)>,
    rules: Vec<Rule<T, M>>,

    tokens: VecDeque<T>,

    start_mode: M,
    mode: M,

    active: bool,
}

struct Rule<T: Token, M> {
    kind: T::Kind,
    matcher: Matcher,
    mode: M,
    mode_to: Option<M>,
}

impl<T, M> Lexer<T, M>
where
    T: Token,
    M: Eq + Copy + Sized + Default,
{
    pub fn new() -> Self {
        Lexer {
            span: VecDeque::new(),
            cursor: 0,
            position: 0,
            pending: None,
            rules: Vec::new(),
            tokens: VecDeque::new(),
            start_mode: M::default(),
            mode: M::default(),
            active: false,
        }
    }

    pub fn add_rule(&mut self, kind: T::Kind, regex: &Regex, mode: M, mode_to: Option<M>) {
        if self.active {
            self.reset();
        }

        let matcher = Matcher::from_regex(&regex.clone());

        let rule = Rule {
            kind,
            matcher,
            mode,
            mode_to,
        };

        self.rules.push(rule);
    }

    pub fn set_start_mode(&mut self, mode: M) {
        if self.active {
            self.reset();
        }

        self.start_mode = mode;
    }

    pub fn reset(&mut self) {
        self.span.clear();
        self.cursor = 0;
        self.position = 0;
        self.pending = None;
        self.tokens.clear();
        self.mode = self.start_mode;

        for rule in &mut self.rules {
            rule.matcher.reset();
        }
    }

    pub fn put(&mut self, c: char) {
        self.active = true;

        self.span.push_front(c);
        self.lex();
    }

    pub fn finish(&mut self) {
        self.emit();
    }

    fn lex(&mut self) {
        while self.cursor < self.span.len() {
            let c = self.span[self.cursor];
            self.cursor += 1;

            let mut accepting_rule = None;
            let mut all_dead = true;

            for (index, rule) in self.rules.iter_mut().enumerate() {
                if self.mode != rule.mode {
                    continue;
                }

                rule.matcher.put(c);

                if rule.matcher.is_accepting() {
                    accepting_rule.get_or_insert(index);
                }

                all_dead &= rule.matcher.is_dead();
            }

            if all_dead {
                self.emit();

                for rule in &mut self.rules {
                    rule.matcher.reset();
                }
            } else if let Some(index) = accepting_rule {
                self.pending = Some((index, self.cursor));
            }
        }
    }

    fn emit(&mut self) {
        if let Some((index, position)) = self.pending.take() {
            let rule = &self.rules[index];

            self.cursor = position;

            let token = T::create(
                rule.kind,
                self.position,
                &self.span.drain(..self.cursor).collect(),
            );

            self.position += self.cursor;
            self.cursor = 0;
            self.mode = rule.mode_to.unwrap_or(self.mode);
            self.tokens.push_front(token);
        } else {
            todo!("syntax error")
        }
    }
}
