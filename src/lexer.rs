use std::collections::VecDeque;

use crate::matcher::Matcher;

pub trait Token: Sized {
    type Kind: Sized + Copy;

    fn create(kind: Self::Kind, position: usize, span: &String) -> Self;
}

pub struct Lexer<T: Token> {
    span: VecDeque<char>,
    cursor: usize,

    position: usize,

    pending: Option<(usize, usize)>,
    rules: Vec<Rule<T>>,

    tokens: VecDeque<T>,
    mode: u64,
}

struct Rule<T: Token> {
    kind: T::Kind,
    matcher: Matcher,
    mode: u64,
    mode_to: Option<u64>,
}

impl<T> Lexer<T>
where
    T: Token,
{
    pub fn put(&mut self, c: char) {
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
