pub enum Token {
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,

    Quote,
    Backquote,
    Comma,

    String(String),
    Atom(String),
}

pub struct Lexeme {
    pub token: Token,

    pub position: usize,
    pub length: usize,
}

enum LexerMode {
    Normal,
    String,
    StringEscape,
    Comment,
    Atom,
}

pub struct Lexer {
    mode: LexerMode,
    buffer: String,
    position: usize,
    output: Vec<Lexeme>,
    error: Option<String>,
}

impl Lexer {
    pub fn new() -> Lexer {
        Lexer {
            mode: LexerMode::Normal,
            buffer: String::new(),
            position: 0,
            output: Vec::new(),
            error: None,
        }
    }

    pub fn update(&mut self, input: char) {
        self.lex(input);
    }

    pub fn finish(&mut self) {
        if self.error.is_some() {
            return;
        }

        match self.mode {
            LexerMode::Normal => {}

            LexerMode::String => {
                self.error = Some("Unterminated string".to_string());
            }

            LexerMode::StringEscape => {
                self.error = Some("Unterminated string escape".to_string());
            }

            LexerMode::Comment => {}

            LexerMode::Atom => {
                self.push_token(Token::Atom(self.buffer.clone()));
            }
        }
    }

    pub fn take(&mut self, lexemes: &mut Vec<Lexeme>) {
        lexemes.extend(self.output.drain(..));
    }

    pub fn is_error(&self) -> bool {
        self.error.is_some()
    }

    pub fn get_error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    fn push_token(&mut self, token: Token) {
        self.output.push(Lexeme {
            token,
            position: self.position,
            length: self.buffer.len(),
        });

        self.position += self.buffer.len();
        self.buffer.clear();
        self.mode = LexerMode::Normal;
    }

    fn newline(&mut self) {
        //
    }

    fn lex(&mut self, input: char) {
        if self.error.is_some() {
            return;
        }

        match self.mode {
            LexerMode::Normal => self.lex_normal(input),
            LexerMode::String => self.lex_string(input),
            LexerMode::StringEscape => self.lex_string_escape(input),
            LexerMode::Comment => self.lex_comment(input),
            LexerMode::Atom => self.lex_atom(input),
        }
    }

    fn lex_normal(&mut self, input: char) {
        match input {
            '(' => self.push_token(Token::LParen),
            ')' => self.push_token(Token::RParen),
            '[' => self.push_token(Token::LBracket),
            ']' => self.push_token(Token::RBracket),
            '{' => self.push_token(Token::LBrace),
            '}' => self.push_token(Token::RBrace),

            '\'' => self.push_token(Token::Quote),
            '`' => self.push_token(Token::Backquote),
            ',' => self.push_token(Token::Comma),

            '"' => {
                self.mode = LexerMode::String;
                self.buffer.clear();
            }

            ';' => self.mode = LexerMode::Comment,

            ' ' | '\t' => {}

            '\n' => self.newline(),

            _ => {
                self.mode = LexerMode::Atom;
                self.buffer.clear();
                self.buffer.push(input);
            }
        }
    }

    fn lex_string(&mut self, input: char) {
        match input {
            '"' => {
                self.push_token(Token::String(self.buffer.clone()));
            }

            '\\' => {
                self.mode = LexerMode::StringEscape;
                self.buffer.push(input);
            }

            '\n' => {
                self.newline();
                self.buffer.push(input);
            }

            _ => {
                self.buffer.push(input);
            }
        }
    }

    fn lex_comment(&mut self, input: char) {
        match input {
            '\n' => {
                self.mode = LexerMode::Normal;
                self.newline();
            }

            _ => {}
        }
    }

    fn lex_atom(&mut self, input: char) {
        match input {
            '(' | ')' | '[' | ']' | '{' | '}' | '\'' | '`' | ',' | '"' | ';' | ' ' | '\t'
            | '\n' => {
                self.push_token(Token::Atom(self.buffer.clone()));
                self.mode = LexerMode::Normal;
                self.lex(input);
            }

            _ => {
                self.buffer.push(input);
            }
        }
    }

    fn lex_string_escape(&mut self, input: char) {
        match input {
            '"' => {
                self.buffer.push(input);
                self.mode = LexerMode::String;
            }

            _ => {
                self.error = Some("Invalid string escape".to_string());
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    fn lex_input(input: &str) -> Vec<Lexeme> {
        let mut lexer = Lexer::new();
        for c in input.chars() {
            lexer.update(c);
        }
        lexer.finish();
        let mut lexemes = Vec::new();
        lexer.take(&mut lexemes);
        lexemes
    }

    #[test]
    fn test_empty_input() {
        let lexemes = lex_input("");
        assert!(lexemes.is_empty());
    }

    #[test]
    fn test_single_parens() {
        let lexemes = lex_input("()");
        assert_eq!(lexemes.len(), 2);
        assert!(matches!(lexemes[0].token, Token::LParen));
        assert!(matches!(lexemes[1].token, Token::RParen));
    }

    #[test]
    fn test_single_brackets() {
        let lexemes = lex_input("[]");
        assert_eq!(lexemes.len(), 2);
        assert!(matches!(lexemes[0].token, Token::LBracket));
        assert!(matches!(lexemes[1].token, Token::RBracket));
    }

    #[test]
    fn test_single_braces() {
        let lexemes = lex_input("{}");
        assert_eq!(lexemes.len(), 2);
        assert!(matches!(lexemes[0].token, Token::LBrace));
        assert!(matches!(lexemes[1].token, Token::RBrace));
    }

    #[test]
    fn test_quotes() {
        let lexemes = lex_input("'`");
        assert_eq!(lexemes.len(), 2);
        assert!(matches!(lexemes[0].token, Token::Quote));
        assert!(matches!(lexemes[1].token, Token::Backquote));
    }

    #[test]
    fn test_comma() {
        let lexemes = lex_input(",");
        assert_eq!(lexemes.len(), 1);
        assert!(matches!(lexemes[0].token, Token::Comma));
    }

    #[test]
    fn test_string() {
        let lexemes = lex_input("\"hello\"");
        assert_eq!(lexemes.len(), 1);
        assert!(matches!(lexemes[0].token, Token::String(ref s) if s == "hello"));
    }

    #[test]
    fn test_atom() {
        let lexemes = lex_input("atom");
        assert_eq!(lexemes.len(), 1);
        assert!(matches!(lexemes[0].token, Token::Atom(ref s) if s == "atom"));
    }

    #[test]
    fn test_mixed_input() {
        let lexemes = lex_input("(atom \"string\" ' ` ,)");
        assert_eq!(lexemes.len(), 7);
        assert!(matches!(lexemes[0].token, Token::LParen));
        assert!(matches!(lexemes[1].token, Token::Atom(ref s) if s == "atom"));
        assert!(matches!(lexemes[2].token, Token::String(ref s) if s == "string"));
        assert!(matches!(lexemes[3].token, Token::Quote));
        assert!(matches!(lexemes[4].token, Token::Backquote));
        assert!(matches!(lexemes[5].token, Token::Comma));
        assert!(matches!(lexemes[6].token, Token::RParen));
    }

    #[test]
    fn test_comment() {
        let lexemes = lex_input("; this is a comment\natom");
        assert_eq!(lexemes.len(), 1);
        assert!(matches!(lexemes[0].token, Token::Atom(ref s) if s == "atom"));
    }

    #[test]
    fn test_string_with_escape() {
        let lexemes = lex_input("\"hello \\\"world\\\"\"");
        assert_eq!(lexemes.len(), 1);
        assert!(matches!(lexemes[0].token, Token::String(ref s) if s == "hello \\\"world\\\""));
    }

    #[test]
    fn test_unterminated_string() {
        let lexemes = lex_input("\"unterminated");
        assert!(lexemes.is_empty());
        let mut lexer = Lexer::new();
        for c in "\"unterminated".chars() {
            lexer.update(c);
        }
        lexer.finish();
        assert!(lexer.is_error());
        assert_eq!(lexer.get_error(), Some("Unterminated string"));
    }

    #[test]
    fn test_unterminated_string_escape() {
        let lexemes = lex_input("\"unterminated\\");
        assert!(lexemes.is_empty());
        let mut lexer = Lexer::new();
        for c in "\"unterminated\\".chars() {
            lexer.update(c);
        }
        lexer.finish();
        assert!(lexer.is_error());
        assert_eq!(lexer.get_error(), Some("Unterminated string escape"));
    }

    #[test]
    fn test_invalid_string_escape() {
        let lexemes = lex_input("\"invalid\\escape\"");
        assert!(lexemes.is_empty());
        let mut lexer = Lexer::new();
        for c in "\"invalid\\escape\"".chars() {
            lexer.update(c);
        }
        lexer.finish();
        assert!(lexer.is_error());
        assert_eq!(lexer.get_error(), Some("Invalid string escape"));
    }

    #[test]
    fn test_unterminated_atom() {
        let lexemes = lex_input("atom(");
        assert_eq!(lexemes.len(), 2);
        assert!(matches!(lexemes[0].token, Token::Atom(ref s) if s == "atom"));
        assert!(matches!(lexemes[1].token, Token::LParen));
    }

    #[test]
    fn test_unterminated_comment() {
        let lexemes = lex_input("; this is an unterminated comment");
        assert!(lexemes.is_empty());
        let mut lexer = Lexer::new();
        for c in "; this is an unterminated comment".chars() {
            lexer.update(c);
        }
        lexer.finish();
        assert!(!lexer.is_error());
    }

    #[test]
    fn test_lexeme_positions() {
        let input = "(atom \"string\" ' ` ,)";
        let lexemes = lex_input(input);
        assert_eq!(lexemes.len(), 7);

        let expected_positions = vec![0, 1, 6, 14, 16, 18, 19];
        for (lexeme, &expected_position) in lexemes.iter().zip(expected_positions.iter()) {
            assert_eq!(lexeme.position, expected_position);
        }
    }

    #[test]
    fn test_large_input() {
        let input = r#"
        (define (factorial n)
            (if (<= n 1)
                1
                (* n (factorial (- n 1)))))
        (define (fibonacci n)
            (if (<= n 1)
                n ;; commenting here too
                (+ (fibonacci (- n 1)) (fibonacci (- n 2)))))
        "This is a large string with multiple lines
        and various structures including (parentheses), [brackets], {braces},
        'quotes', `backquotes`, and ,commas."
        ; This is a comment
        (print "Done")
        "#;
        let lexemes = lex_input(input);
        assert!(!lexemes.is_empty());

        for lexeme in lexemes {
            if let Token::Atom(ref s) = lexeme.token {
                assert!(!s.contains(' '), "Atom token contains whitespace: {}", s);
            }
        }
    }
}
