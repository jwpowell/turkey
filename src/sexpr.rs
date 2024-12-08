use std::default;

use crate::lexer::{Lexer, Token};
use crate::regex::Regex;

pub fn lexer() -> Lexer<SexprToken, SexprMode> {
    use SexprMode::*;
    use SexprTokenKind::*;

    let mut lexer = Lexer::new();

    lexer.add_rule(LParen, &Regex::symbol('('), Default, None);
    lexer.add_rule(RParen, &Regex::symbol(')'), Default, None);
    lexer.add_rule(LBrace, &Regex::symbol('{'), Default, None);
    lexer.add_rule(RBrace, &Regex::symbol('}'), Default, None);
    lexer.add_rule(LBracket, &Regex::symbol('['), Default, None);
    lexer.add_rule(RBracket, &Regex::symbol(']'), Default, None);

    let regex_alpha = Regex::range('a', 'z').union(&Regex::range('A', 'Z'));
    let regex_digit = Regex::range('0', '9');
    let regex_alphanumeric = regex_alpha.union(&regex_digit);

    let regex_identifier_start = regex_alpha.union(&Regex::any_of("_~!$%^&*_=:"));
    let regex_identifier_continue = regex_identifier_start
        .union(&regex_digit)
        .union(&Regex::any_of("-+"));

    let regex_identifier = regex_identifier_start.concat(&regex_identifier_continue.star());

    lexer.add_rule(Identifier, &regex_identifier, Default, None);

    let regex_string = todo!();

    lexer.add_rule(String, &regex_string, Default, None);

    let integer = todo!();

    lexer.add_rule(Integer, &integer, Default, None);

    let float = todo!();

    lexer.add_rule(Float, &float, Default, None);

    let comment = todo!();

    lexer.add_rule(SexprTokenKind::Comment, &comment, Default, None);

    let whitespace = todo!();

    lexer.add_rule(Whitespace, &whitespace, Default, None);

    let newline = todo!();

    lexer.add_rule(Newline, &newline, Default, None);

    todo!();
    lexer
}

pub enum SexprToken {
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Identifier(String),
    String(String),
    Integer(i64),
    Float(f64),
    Comment(String),
    Whitespace,
    Newline,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum SexprTokenKind {
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Identifier,
    String,
    Integer,
    Float,
    Comment,
    Whitespace,
    Newline,
}

#[derive(Copy, Clone, PartialEq, Eq, Default)]
pub enum SexprMode {
    #[default]
    Default,
    Comment,
}

impl Token for SexprToken {
    type Kind = SexprTokenKind;

    fn create(kind: Self::Kind, _position: usize, span: &String) -> Self {
        use SexprTokenKind::*;

        match kind {
            LParen => SexprToken::LParen,
            RParen => SexprToken::RParen,
            LBrace => SexprToken::LBrace,
            RBrace => SexprToken::RBrace,
            LBracket => SexprToken::LBracket,
            RBracket => SexprToken::RBracket,
            Identifier => SexprToken::Identifier(span.clone()),
            String => SexprToken::String(span.clone()),
            Integer => SexprToken::Integer(span.parse().unwrap()),
            Float => SexprToken::Float(span.parse().unwrap()),
            Comment => SexprToken::Comment(span.clone()),
            Whitespace => SexprToken::Whitespace,
            Newline => SexprToken::Newline,
        }
    }
}
