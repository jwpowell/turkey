use std::io::BufRead;

use crate::lex::lexer::*;
use crate::lex::regex::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
enum Mode {
    #[default]
    Default,
    Comment,
    String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Token {
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,

    Semicolon,
    Comma,
    Quote,
    BackQuote,
    DoubleQuote,

    Whitespace,
    Newline,

    Integer,
    Float,
    Identifier,
    String,
    StringEscape,

    Comment,
}

pub struct Compiler {
    lexer: Lexer<Mode, Token>,
}

impl Compiler {
    pub fn new() -> Self {
        let mut compiler = Compiler {
            lexer: Lexer::new(),
        };

        compiler.lexer_init();

        compiler
    }

    pub fn lex<R>(&mut self, mut io: R, lexemes: &mut Vec<Lexeme<Token>>)
    where
        R: BufRead,
    {
        let mut buf = vec![];

        while io.read_until(b'\n', &mut buf).unwrap() != 0 {
            let s = String::from_utf8(buf).unwrap();

            for c in s.chars() {
                self.lexer.put(c);

                if let Some(error) = self.lexer.get_error() {
                    let error = error.clone();
                    while let Some(lexeme) = self.lexer.get() {
                        lexemes.push(lexeme.clone())
                    }
                    panic!("error: {:?}", error);
                }
            }

            buf = s.into_bytes();
            buf.clear();
        }

        self.lexer.finish();

        while let Some(lexeme) = self.lexer.get() {
            lexemes.push(lexeme.clone())
        }
    }

    fn lexer_init(&mut self) {
        use Token::*;

        self.lexer = Lexer::new();

        let lparen = Regex::char('(');
        let rparen = Regex::char(')');
        let lbrace = Regex::char('{');
        let rbrace = Regex::char('}');
        let lbracket = Regex::char('[');
        let rbracket = Regex::char(']');
        let semicolon = Regex::char(';');
        let comma = Regex::char(',');
        let quote = Regex::char('\'');
        let backquote = Regex::char('`');
        let doublequote = Regex::char('"');

        let whitespace = Regex::one_of(" \t").star();
        let newline = Regex::char('\n');

        let digit = Regex::range('0', '9');
        let sign = Regex::one_of("+-");
        let alpha_lower = Regex::range('a', 'z');
        let alpha_upper = Regex::range('A', 'Z');
        let alpha = alpha_lower.union(&alpha_upper);
        let alphanumeric = alpha.union(&digit);
        let ident_symbols = Regex::none_of("(){}[];,'\" \t\n`0123456789");

        let ident_start = alpha.union(&ident_symbols);
        let ident_continue = alphanumeric.union(&ident_symbols);
        let identifier = ident_start.concat(&ident_continue.star());

        let integer = sign.optional().concat(&digit.plus());
        let exponent = Regex::one_of("eE").concat(&integer);
        let float_exp = integer.concat(&exponent);
        let float_frac = integer
            .concat(&Regex::char('.'))
            .concat(&integer)
            .concat(&exponent.optional());
        let float = float_frac.union(&float_exp);

        let string = Regex::none_of("\"\\").star();
        let string_escape = Regex::char('\\').concat(&Regex::any());

        let comment = Regex::none_of("\n").star();

        // Normal mode rules
        self.lexer
            .with_rule(LParen, &lparen, Mode::Default, Mode::Default, false)
            .with_rule(RParen, &rparen, Mode::Default, Mode::Default, false)
            .with_rule(LBrace, &lbrace, Mode::Default, Mode::Default, false)
            .with_rule(RBrace, &rbrace, Mode::Default, Mode::Default, false)
            .with_rule(LBracket, &lbracket, Mode::Default, Mode::Default, false)
            .with_rule(RBracket, &rbracket, Mode::Default, Mode::Default, false)
            .with_rule(Semicolon, &semicolon, Mode::Default, Mode::Comment, false)
            .with_rule(Comma, &comma, Mode::Default, Mode::Default, false)
            .with_rule(Quote, &quote, Mode::Default, Mode::Default, false)
            .with_rule(BackQuote, &backquote, Mode::Default, Mode::Default, false)
            .with_rule(
                DoubleQuote,
                &doublequote,
                Mode::Default,
                Mode::String,
                false,
            )
            .with_rule(Whitespace, &whitespace, Mode::Default, Mode::Default, false)
            .with_rule(Newline, &newline, Mode::Default, Mode::Default, false)
            .with_rule(Integer, &integer, Mode::Default, Mode::Default, true)
            .with_rule(Float, &float, Mode::Default, Mode::Default, true)
            .with_rule(Identifier, &identifier, Mode::Default, Mode::Default, true);

        self.lexer
            .with_rule(String, &string, Mode::String, Mode::String, true)
            .with_rule(
                StringEscape,
                &string_escape,
                Mode::String,
                Mode::String,
                true,
            )
            .with_rule(
                DoubleQuote,
                &doublequote,
                Mode::String,
                Mode::Default,
                false,
            );

        self.lexer
            .with_rule(Comment, &comment, Mode::Comment, Mode::Comment, false)
            .with_rule(Newline, &newline, Mode::Comment, Mode::Default, false);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_lex_empty_input() {
        let mut compiler = Compiler::new();
        let input = "";
        let mut lexemes = vec![];
        compiler.lex(Cursor::new(input), &mut lexemes);
        assert!(lexemes.is_empty());
    }

    #[test]
    fn test_lex_single_token() {
        let mut compiler = Compiler::new();
        let input = "(";
        let mut lexemes = vec![];
        compiler.lex(Cursor::new(input), &mut lexemes);
        assert_eq!(lexemes.len(), 1);
        assert_eq!(lexemes[0].token, Token::LParen);
    }

    #[test]
    fn test_lex_multiple_tokens() {
        let mut compiler = Compiler::new();
        let input = "( ) { }";
        let mut lexemes = vec![];
        compiler.lex(Cursor::new(input), &mut lexemes);
        assert_eq!(lexemes.len(), 7);
        assert_eq!(lexemes[0].token, Token::LParen);
        assert_eq!(lexemes[1].token, Token::Whitespace);
        assert_eq!(lexemes[2].token, Token::RParen);
        assert_eq!(lexemes[3].token, Token::Whitespace);
        assert_eq!(lexemes[4].token, Token::LBrace);
        assert_eq!(lexemes[5].token, Token::Whitespace);
        assert_eq!(lexemes[6].token, Token::RBrace);
    }

    #[test]
    fn test_lex_identifier() {
        let mut compiler = Compiler::new();
        let input = "hello";
        let mut lexemes = vec![];
        compiler.lex(Cursor::new(input), &mut lexemes);
        assert_eq!(lexemes.len(), 1);
        assert_eq!(lexemes[0].token, Token::Identifier);
    }

    #[test]
    fn test_lex_integer() {
        let mut compiler = Compiler::new();
        let input = "123";
        let mut lexemes = vec![];
        compiler.lex(Cursor::new(input), &mut lexemes);
        assert_eq!(lexemes.len(), 1);
        assert_eq!(lexemes[0].token, Token::Integer);
    }

    #[test]
    fn test_lex_float() {
        let mut compiler = Compiler::new();
        let input = "123.456";
        let mut lexemes = vec![];

        compiler.lex(Cursor::new(input), &mut lexemes);

        assert_eq!(lexemes.len(), 1);
        assert_eq!(lexemes[0].token, Token::Float);
    }

    #[test]
    fn test_lex_string() {
        let mut compiler = Compiler::new();
        let input = "\"hello\"";
        let mut lexemes = vec![];
        compiler.lex(Cursor::new(input), &mut lexemes);

        assert_eq!(lexemes.len(), 3);
        assert_eq!(lexemes[0].token, Token::DoubleQuote);
        assert_eq!(lexemes[1].token, Token::String);
        assert_eq!(lexemes[2].token, Token::DoubleQuote);
    }

    #[test]
    fn test_lex_comment() {
        let mut compiler = Compiler::new();
        let input = "; this is a comment\n";
        let mut lexemes = vec![];
        compiler.lex(Cursor::new(input), &mut lexemes);
        assert_eq!(lexemes.len(), 3);
        assert_eq!(lexemes[0].token, Token::Semicolon);
        assert_eq!(lexemes[1].token, Token::Comment);
        assert_eq!(lexemes[2].token, Token::Newline);
    }
}
