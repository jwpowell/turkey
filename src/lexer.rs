pub struct Lexer;

impl Lexer {
    pub fn new() -> Lexer {
        todo!()
    }

    pub fn put(&mut self, c: char) {
        todo!()
    }

    pub fn finish(&mut self) {
        todo!()
    }

    pub fn get(&mut self) -> Option<Lexeme> {
        todo!()
    }

    pub fn error(&self) -> Option<String> {
        todo!()
    }
}

pub struct Lexeme {
    token: Token,
    position: usize,
    length: usize,
    span: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Token {
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Quote,
    DoubleQuote,
    Backquote,
    Newline,
    Identifier,
    String,
    StringEscape,
    Integer,
    Float,
}

const ALL_RULES: [Token; 13] = [
    Token::LParen,
    Token::RParen,
    Token::LBrace,
    Token::RBrace,
    Token::LBracket,
    Token::RBracket,
    Token::Comma,
    Token::Quote,
    Token::DoubleQuote,
    Token::Backquote,
    Token::Newline,
    Token::Identifier,
    Token::String,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mode {
    Normal,
    String,
    Comment,
}

const ALL_MODES: [Mode; 3] = [Mode::Normal, Mode::String, Mode::Comment];

pub struct Rule {
    token: Token,
    keep_span: bool,
    regex: Regex,
    nfa: Nfa,
    mode: Mode,
    mode_to: Mode,
}
