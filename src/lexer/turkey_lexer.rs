use crate::lexer::lexer::Lexer;
use crate::lexer::regex::Regex;
use crate::lexer::token::Token;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mode {
    Normal,
    String,
    Comment,
}

pub fn turkey_lexer() -> Lexer<Token, Mode> {
    let mut lexer = Lexer::new();

    lexer.add_rule(
        &Token::LParen.regex(),
        Token::LParen,
        false,
        Mode::Normal,
        Mode::Normal,
    );
    lexer.add_rule(
        &Token::RParen.regex(),
        Token::RParen,
        false,
        Mode::Normal,
        Mode::Normal,
    );
    lexer.add_rule(
        &Token::Quote.regex(),
        Token::Quote,
        false,
        Mode::Normal,
        Mode::Normal,
    );

    lexer.add_rule(
        &Token::Semicolon.regex(),
        Token::Semicolon,
        false,
        Mode::Normal,
        Mode::Comment,
    );

    lexer.add_rule(
        &Token::DoubleQuote.regex(),
        Token::DoubleQuote,
        false,
        Mode::Normal,
        Mode::String,
    );
    lexer.add_rule(
        &Token::Comma.regex(),
        Token::Comma,
        false,
        Mode::Normal,
        Mode::Normal,
    );
    lexer.add_rule(
        &Token::Dot.regex(),
        Token::Dot,
        false,
        Mode::Normal,
        Mode::Normal,
    );
    lexer.add_rule(
        &Token::Semicolon.regex(),
        Token::Semicolon,
        false,
        Mode::Normal,
        Mode::Normal,
    );
    lexer.add_rule(
        &Token::Comment.regex(),
        Token::Comment,
        false,
        Mode::Comment,
        Mode::Normal,
    );
    lexer.add_rule(
        &Token::Whitespace.regex(),
        Token::Whitespace,
        false,
        Mode::Normal,
        Mode::Normal,
    );
    lexer.add_rule(
        &Token::Newline.regex(),
        Token::Newline,
        false,
        Mode::Normal,
        Mode::Normal,
    );
    lexer.add_rule(
        &Token::Integer.regex(),
        Token::Integer,
        false,
        Mode::Normal,
        Mode::Normal,
    );
    lexer.add_rule(
        &Token::Float.regex(),
        Token::Float,
        false,
        Mode::Normal,
        Mode::Normal,
    );
    lexer.add_rule(
        &Token::Identifier.regex(),
        Token::Identifier,
        false,
        Mode::Normal,
        Mode::Normal,
    );

    lexer.add_rule(
        &Token::String.regex(),
        Token::String,
        false,
        Mode::String,
        Mode::Normal,
    );

    lexer.set_start_mode(Mode::Normal);

    lexer
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_large_string() {
        let mut lexer = turkey_lexer();

        let large_string = r#"(define (incr x) (+ x 1.1))
(1 11 +11 -11 +11.0 -11.0 11e10)"#;

        lexer.reset();

        for c in large_string.chars() {
            lexer.put(c);
            println!("{}", c);
            println!("{:?}", lexer.error());
        }

        lexer.finish();

        let mut tokens = Vec::new();
        while let Some(lexeme) = lexer.get() {
            tokens.push(lexeme.token);
        }

        use Token::*;

        assert_eq!(
            tokens,
            vec![
                LParen, Identifier, Whitespace, LParen, Identifier, Whitespace, Identifier, RParen,
                Whitespace, LParen, Identifier, Whitespace, Identifier, Whitespace, Float, RParen,
                RParen, Newline, LParen, Integer, Whitespace, Integer, Whitespace, Integer,
                Whitespace, Integer, Whitespace, Float, Whitespace, Float, Whitespace, Float,
                RParen
            ]
        );
    }
}
