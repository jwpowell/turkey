use super::regex::Regex;

pub enum Token {
    LParen,
    RParen,

    Quote,
    DoubleQuote,
    Comma,
    Dot,
    Semicolon,

    Comment,
    Whitespace,
    Newline,

    Identifier,
    Integer,
    Float,
}

impl Token {
    pub fn regex(&self) -> Regex {
        match self {
            Token::LParen => Self::regex_lparen(),
            Token::RParen => Self::regex_rparen(),
            Token::Identifier => Self::regex_identifier(),
            Token::Integer => Self::regex_integer(),
            Token::Float => Self::regex_float(),
            Token::Quote => Self::regex_quote(),
            Token::DoubleQuote => Self::regex_double_quote(),
            Token::Comma => Self::regex_comma(),
            Token::Dot => Self::regex_dot(),
            Token::Semicolon => Self::regex_semicolon(),
            Token::Comment => Self::regex_comment(),
            Token::Whitespace => Self::regex_whitespace(),
            Token::Newline => Self::regex_newline(),
        }
    }

    fn regex_lparen() -> Regex {
        Regex::char('(')
    }

    fn regex_rparen() -> Regex {
        Regex::char(')')
    }

    fn regex_identifier() -> Regex {
        let ident_start = [
            Regex::range('a', 'z'),
            Regex::range('A', 'Z'),
            Regex::one_of("`~!@#$%^&*-+=|:<>?/"),
        ]
        .iter()
        .fold(Regex::empty(), |acc, r| acc.union(r));

        let ident_continue = [Regex::range('0', '9')]
            .iter()
            .fold(Regex::empty(), |acc, r| acc.union(r))
            .union(&ident_start);

        ident_start.concat(&ident_continue.star())
    }

    fn regex_integer() -> Regex {
        let digit = Regex::range('0', '9');
        let sign = Regex::one_of("-+");

        sign.opt().concat(&digit.plus())
    }

    fn regex_float() -> Regex {
        let integer = Self::regex_integer();
        let dot = Regex::char('.');
        let fraction = integer.clone();
        let exponent = Regex::char('e')
            .union(&Regex::char('E'))
            .concat(&integer.plus());

        integer.concat(&dot.concat(&fraction.opt().concat(&exponent.opt())))
    }

    fn regex_quote() -> Regex {
        Regex::char('\'')
    }

    fn regex_double_quote() -> Regex {
        Regex::char('"')
    }

    fn regex_comma() -> Regex {
        Regex::char(',')
    }

    fn regex_dot() -> Regex {
        Regex::char('.')
    }

    fn regex_semicolon() -> Regex {
        Regex::char(';')
    }

    fn regex_comment() -> Regex {
        Regex::none_of("\n").star()
    }

    fn regex_whitespace() -> Regex {
        Regex::one_of(" \t")
    }

    fn regex_newline() -> Regex {
        Regex::char('\n')
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::lexer::regex::Matcher;

    #[test]
    fn test_token_regex_lparen() {
        let regex = Token::LParen.regex();
        let mut matcher = Matcher::new(&regex);

        assert!(matcher.match_string("("));
        assert!(!matcher.match_string(")"));
    }

    #[test]
    fn test_token_regex_rparen() {
        let regex = Token::RParen.regex();
        let mut matcher = Matcher::new(&regex);

        assert!(matcher.match_string(")"));
        assert!(!matcher.match_string("("));
    }

    #[test]
    fn test_token_regex_identifier() {
        let regex = Token::Identifier.regex();
        let mut matcher = Matcher::new(&regex);

        assert!(matcher.match_string("a"));
        assert!(matcher.match_string("abc"));
        assert!(matcher.match_string("a1b2c3"));
        assert!(!matcher.match_string("1abc"));
    }

    #[test]
    fn test_token_regex_integer() {
        let regex = Token::Integer.regex();
        let mut matcher = Matcher::new(&regex);

        assert!(matcher.match_string("123"));
        assert!(matcher.match_string("-123"));
        assert!(matcher.match_string("+123"));
        assert!(!matcher.match_string("123.456"));
    }

    #[test]
    fn test_token_regex_float() {
        let regex = Token::Float.regex();
        let mut matcher = Matcher::new(&regex);

        assert!(matcher.match_string("123.456"));
        assert!(matcher.match_string("-123.456"));
        assert!(matcher.match_string("+123.456"));
        assert!(matcher.match_string("123.456e789"));
        assert!(!matcher.match_string("123"));
    }

    #[test]
    fn test_token_regex_quote() {
        let regex = Token::Quote.regex();
        let mut matcher = Matcher::new(&regex);

        assert!(matcher.match_string("'"));
        assert!(!matcher.match_string("\""));
    }

    #[test]
    fn test_token_regex_double_quote() {
        let regex = Token::DoubleQuote.regex();
        let mut matcher = Matcher::new(&regex);

        assert!(matcher.match_string("\""));
        assert!(!matcher.match_string("'"));
    }

    #[test]
    fn test_token_regex_comma() {
        let regex = Token::Comma.regex();
        let mut matcher = Matcher::new(&regex);

        assert!(matcher.match_string(","));
        assert!(!matcher.match_string("."));
    }

    #[test]
    fn test_token_regex_dot() {
        let regex = Token::Dot.regex();
        let mut matcher = Matcher::new(&regex);

        assert!(matcher.match_string("."));
        assert!(!matcher.match_string(","));
    }

    #[test]
    fn test_token_regex_semicolon() {
        let regex = Token::Semicolon.regex();
        let mut matcher = Matcher::new(&regex);

        assert!(matcher.match_string(";"));
        assert!(!matcher.match_string(":"));
    }

    #[test]
    fn test_token_regex_comment() {
        let regex = dbg!(Token::Comment.regex());
        let mut matcher = Matcher::new(&regex);

        assert!(matcher.match_string("This is a comment"));
        assert!(!matcher.match_string("\n"));
    }

    #[test]
    fn test_token_regex_whitespace() {
        let regex = Token::Whitespace.regex();
        let mut matcher = Matcher::new(&regex);

        assert!(matcher.match_string(" "));
        assert!(matcher.match_string("\t"));
        assert!(!matcher.match_string("\n"));
    }

    #[test]
    fn test_token_regex_newline() {
        let regex = Token::Newline.regex();
        let mut matcher = Matcher::new(&regex);

        assert!(matcher.match_string("\n"));
        assert!(!matcher.match_string(" "));
    }
}
