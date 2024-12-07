use rand::distributions::{Alphanumeric, DistString};

use rand::seq::SliceRandom;
use turkey::sexpr::*;

fn token_to_string(token: &Token) -> String {
    match token {
        Token::LParen => "(".to_string(),
        Token::RParen => ")".to_string(),
        Token::LBracket => "[".to_string(),
        Token::RBracket => "]".to_string(),
        Token::LBrace => "{".to_string(),
        Token::RBrace => "}".to_string(),
        Token::Quote => "'".to_string(),
        Token::Backquote => "`".to_string(),
        Token::Comma => ",".to_string(),
        Token::String(s) => format!("\"{}\"", s),
        Token::Identifier(s) => s.clone(),
        Token::Integer(s) => s.clone(),
        Token::Float(s) => s.clone(),
    }
}

#[test]
fn randomized_action() {
    let space: [Token; 29] = [
        Token::LParen,
        Token::RParen,
        Token::LBracket,
        Token::RBracket,
        Token::LBrace,
        Token::RBrace,
        Token::Quote,
        Token::Backquote,
        Token::Comma,
        Token::String("string".to_string()),
        Token::Identifier("identifier".to_string()),
        Token::Integer("1234".to_string()),
        Token::Integer("+1234".to_string()),
        Token::Integer("-1234".to_string()),
        Token::Float("1234.1234".to_string()),
        Token::Float("+1234.1234".to_string()),
        Token::Float("-1234.1234".to_string()),
        Token::Float("1e10".to_string()),
        Token::Float("+1e10".to_string()),
        Token::Float("-1e10".to_string()),
        Token::Float("1.0e10".to_string()),
        Token::Float("+1.0e10".to_string()),
        Token::Float("-1.0e10".to_string()),
        Token::Float("1.0e-10".to_string()),
        Token::Float("+1.0e-10".to_string()),
        Token::Float("-1.0e-10".to_string()),
        Token::Float("1.0e+10".to_string()),
        Token::Float("+1.0e+10".to_string()),
        Token::Float("-1.0e+10".to_string()),
    ];

    let mut rng = rand::thread_rng();
    let mut source = String::new();

    let mut expected = Vec::new();
    let mut actual = Vec::new();

    const TRIALS: usize = 20;
    const LENGTH: usize = 1_000;

    for _trial in 0..TRIALS {
        actual.clear();
        expected.clear();

        source.clear();

        let mut lexer = Lexer::new();

        for _ in 0..LENGTH {
            let token = space.choose(&mut rng).unwrap();
            let s = token_to_string(token);

            source.push_str(&s);
            source.push('(');
            for c in s.chars() {
                lexer.update(c);
            }
            lexer.update('(');

            expected.push(token.clone());
            expected.push(Token::LParen);
        }
        lexer.finish();

        for _ in 0..source.len() {
            lexer.take(&mut actual);
        }

        assert_eq!(
            expected.iter().collect::<Vec<&Token>>(),
            actual.iter().map(|x| &x.token).collect::<Vec<&Token>>(),
            "Source: \n{}\n",
            source
        );
    }
}
