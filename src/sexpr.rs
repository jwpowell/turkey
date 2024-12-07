#[derive(Debug)]
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
    Identifier(String),
    Integer(String),
    Float(String),
}

#[derive(Debug)]
pub struct Lexeme {
    pub token: Token,

    pub position: usize,
    pub length: usize,

    pub line: usize,
    pub column: usize,
}

#[derive(Debug)]
enum LexerMode {
    Normal,
    String,
    StringEscape,
    Comment,
    Identifier,
    Integer,
    Float,
    FloatExpStart,
    FloatExp,
    EndOfStream,
}

#[derive(Debug, Clone, Copy)]
struct Location {
    position: usize,
    column: usize,
    line: usize,
}

impl Location {
    fn beginning() -> Location {
        Location {
            position: 0,
            column: 1,
            line: 1,
        }
    }
}

pub struct Lexer {
    mode: LexerMode,
    buffer: String,

    current: Location,
    start: Location,

    output: Vec<Lexeme>,
    error: Option<String>,
}

impl Lexer {
    pub fn new() -> Lexer {
        Lexer {
            mode: LexerMode::Normal,
            buffer: String::new(),

            current: Location::beginning(),
            start: Location::beginning(),

            output: Vec::new(),
            error: None,
        }
    }

    pub fn update(&mut self, input: char) {
        self.lex(Some(input));
    }

    pub fn finish(&mut self) {
        self.lex(None);
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

    fn emit(&mut self, token: Token, should_advance: bool) {
        print!("  emit({:?}, {})\n    ", token, should_advance);
        let length = self.buffer.len();
        let lexeme = Lexeme {
            token,
            position: self.start.position,
            length,
            line: self.start.line,
            column: self.start.column,
        };
        self.output.push(lexeme);

        if should_advance {
            self.advance();
        }

        self.commit();
    }

    fn skip(&mut self) {
        print!("  skip()\n    ");
        self.advance();

        print!("    ");
        self.commit();
    }

    fn save_and_advance(&mut self, input: char) {
        print!("  save_and_advance({:?})\n    ", input);
        self.buffer.push(input);
        self.advance();
    }

    fn advance(&mut self) {
        println!("  advance()");
        self.current.position += 1;
        self.current.column += 1;
    }

    fn commit(&mut self) {
        println!("  commit()");

        self.start = self.current;
        self.buffer.clear();
    }

    fn newline(&mut self) {
        println!("  newline()");

        self.current.line += 1;
        self.current.column = 1;
    }

    fn change_modes(&mut self, mode: LexerMode) {
        self.mode = mode;
    }

    fn lex(&mut self, input: Option<char>) {
        println!("lex({:?})", input);
        println!("  mode: {:?}", self.mode);
        println!("  buffer: {:?}", self.buffer);
        println!("  current: {:?}", self.current);
        println!("  start: {:?}", self.start);

        if self.error.is_some() {
            return;
        }

        match self.mode {
            LexerMode::Normal => self.lex_normal(input),
            LexerMode::String => self.lex_string(input),
            LexerMode::StringEscape => self.lex_string_escape(input),
            LexerMode::Comment => self.lex_comment(input),
            LexerMode::Identifier => self.lex_identifier(input),
            LexerMode::Integer => self.lex_integer(input),
            LexerMode::Float => self.lex_float(input),
            LexerMode::FloatExpStart => self.lex_float_exp_start(input),
            LexerMode::FloatExp => self.lex_float_exp(input),

            LexerMode::EndOfStream => {
                panic!("Lexer should not be called after end of stream");
            }
        }
    }

    fn lex_normal(&mut self, input: Option<char>) {
        match input {
            Some('(') => self.emit(Token::LParen, true),
            Some(')') => self.emit(Token::RParen, true),
            Some('[') => self.emit(Token::LBracket, true),
            Some(']') => self.emit(Token::RBracket, true),
            Some('{') => self.emit(Token::LBrace, true),
            Some('}') => self.emit(Token::RBrace, true),

            Some('\'') => self.emit(Token::Quote, true),
            Some('`') => self.emit(Token::Backquote, true),
            Some(',') => self.emit(Token::Comma, true),

            Some('"') => {
                self.advance();
                self.change_modes(LexerMode::String);
            }

            Some(';') => {
                self.skip();
                self.change_modes(LexerMode::Comment);
            }

            Some(' ') | Some('\t') => self.skip(),

            Some('\n') => {
                self.skip();
                self.newline();
                self.commit();
            }

            None => {
                self.change_modes(LexerMode::EndOfStream);
            }

            Some('-') | Some('0'..='9') => {
                self.save_and_advance(input.unwrap());
                self.change_modes(LexerMode::Integer);
            }

            Some(c) => {
                self.save_and_advance(c);
                self.change_modes(LexerMode::Identifier);
            }
        }
    }

    fn lex_string(&mut self, input_opt: Option<char>) {
        match input_opt {
            Some('"') => {
                self.emit(Token::String(self.buffer.clone()), true);
                self.change_modes(LexerMode::Normal);
            }

            Some(c @ '\\') => {
                self.save_and_advance(c);
                self.change_modes(LexerMode::StringEscape);
            }

            Some('\n') => {
                self.save_and_advance('\n');
                self.newline();
            }

            Some(c) => {
                self.save_and_advance(c);
            }

            None => {
                self.error = Some("Unterminated string".to_string());
            }
        }
    }

    fn lex_comment(&mut self, input: Option<char>) {
        match input {
            Some('\n') => {
                self.skip();
                self.newline();
                self.change_modes(LexerMode::Normal);
            }

            _ => {
                self.skip();
            }
        }
    }

    fn lex_identifier(&mut self, input_opt: Option<char>) {
        match input_opt {
            Some('(') | Some(')') | Some('[') | Some(']') | Some('{') | Some('}') | Some('\'')
            | Some('`') | Some(',') | Some('"') | Some(';') | Some(' ') | Some('\t')
            | Some('\n') => {
                self.emit(Token::Identifier(self.buffer.clone()), false);

                self.change_modes(LexerMode::Normal);
                self.lex(input_opt);
            }

            Some(c) => {
                self.save_and_advance(c);
            }

            None => {
                self.emit(Token::Identifier(self.buffer.clone()), true);
            }
        }
    }

    fn lex_string_escape(&mut self, input_opt: Option<char>) {
        match input_opt {
            Some('"') => {
                self.buffer.push('"');
                self.mode = LexerMode::String;
            }

            Some(_) => {
                self.error = Some("Invalid string escape".to_string());
            }

            None => {
                self.error = Some("Unterminated string escape".to_string());
            }
        }
    }

    fn lex_integer(&mut self, input_opt: Option<char>) {
        match input_opt {
            Some(c @ '0'..='9') => {
                self.save_and_advance(c);
            }

            Some('.') => {
                self.save_and_advance('.');
                self.change_modes(LexerMode::Float);
            }

            Some('e') | Some('E') => {
                self.save_and_advance('e');
                self.change_modes(LexerMode::FloatExpStart);
            }

            Some('(') | Some(')') | Some('[') | Some(']') | Some('{') | Some('}') | Some('\'')
            | Some('`') | Some(',') | Some('"') | Some(';') | Some(' ') | Some('\t')
            | Some('\n') | None => {
                self.emit(Token::Integer(self.buffer.clone()), false);

                self.change_modes(LexerMode::Normal);
                self.lex(input_opt);
            }

            _ => {
                self.error = Some("Invalid integer".to_string());
            }
        }
    }

    fn lex_float(&mut self, input_opt: Option<char>) {
        match input_opt {
            Some(c @ '0'..='9') => {
                self.save_and_advance(c);
            }

            Some('e') | Some('E') => {
                self.save_and_advance('e');
                self.change_modes(LexerMode::FloatExpStart);
            }

            Some('(') | Some(')') | Some('[') | Some(']') | Some('{') | Some('}') | Some('\'')
            | Some('`') | Some(',') | Some('"') | Some(';') | Some(' ') | Some('\t')
            | Some('\n') | None => {
                self.emit(Token::Float(self.buffer.clone()), false);

                self.change_modes(LexerMode::Normal);
                self.lex(input_opt);
            }

            _ => {
                self.error = Some("Invalid float".to_string());
            }
        }
    }

    fn lex_float_exp_start(&mut self, input_opt: Option<char>) {
        match input_opt {
            Some(c @ '0'..='9') | Some(c @ '+') | Some(c @ '-') => {
                self.save_and_advance(c);
                self.change_modes(LexerMode::FloatExp);
            }

            Some('(') | Some(')') | Some('[') | Some(']') | Some('{') | Some('}') | Some('\'')
            | Some('`') | Some(',') | Some('"') | Some(';') | Some(' ') | Some('\t')
            | Some('\n') | None => {
                self.error = Some("Invalid float exponent start".to_string());
            }

            _ => {
                self.error = Some("Invalid float exponent".to_string());
            }
        }
    }

    fn lex_float_exp(&mut self, input_opt: Option<char>) {
        match input_opt {
            Some(c @ '0'..='9') => {
                self.save_and_advance(c);
            }

            Some('(') | Some(')') | Some('[') | Some(']') | Some('{') | Some('}') | Some('\'')
            | Some('`') | Some(',') | Some('"') | Some(';') | Some(' ') | Some('\t')
            | Some('\n') => {
                self.emit(Token::Float(self.buffer.clone()), false);

                self.change_modes(LexerMode::Normal);
                self.lex(input_opt);
            }

            None => {
                self.emit(Token::Float(self.buffer.clone()), true);
            }

            _ => {
                self.error = Some("Invalid float exponent".to_string());
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
        assert_eq!(lexemes[0].position, 0);
        assert!(matches!(lexemes[1].token, Token::RParen));
        assert_eq!(lexemes[1].position, 1);
    }

    #[test]
    fn test_single_brackets() {
        let lexemes = lex_input("[]");
        assert_eq!(lexemes.len(), 2);
        assert!(matches!(lexemes[0].token, Token::LBracket));
        assert_eq!(lexemes[0].position, 0);
        assert!(matches!(lexemes[1].token, Token::RBracket));
        assert_eq!(lexemes[1].position, 1);
    }

    #[test]
    fn test_single_braces() {
        let lexemes = lex_input("{}");
        assert_eq!(lexemes.len(), 2);
        assert!(matches!(lexemes[0].token, Token::LBrace));
        assert_eq!(lexemes[0].position, 0);
        assert!(matches!(lexemes[1].token, Token::RBrace));
        assert_eq!(lexemes[1].position, 1);
    }

    #[test]
    fn test_quotes() {
        let lexemes = lex_input("'`");
        assert_eq!(lexemes.len(), 2);
        assert!(matches!(lexemes[0].token, Token::Quote));
        assert_eq!(lexemes[0].position, 0);
        assert!(matches!(lexemes[1].token, Token::Backquote));
        assert_eq!(lexemes[1].position, 1);
    }

    #[test]
    fn test_comma() {
        let lexemes = lex_input(",");
        assert_eq!(lexemes.len(), 1);
        assert!(matches!(lexemes[0].token, Token::Comma));
        assert_eq!(lexemes[0].position, 0);
    }

    #[test]
    fn test_string() {
        let lexemes = lex_input("\"hello\"");
        assert_eq!(lexemes.len(), 1);
        assert!(matches!(lexemes[0].token, Token::String(ref s) if s == "hello"));
        assert_eq!(lexemes[0].position, 0);
    }

    #[test]
    fn test_atom() {
        let lexemes = lex_input("atom");
        assert_eq!(lexemes.len(), 1);
        assert!(matches!(lexemes[0].token, Token::Identifier(ref s) if s == "atom"));
        assert_eq!(lexemes[0].position, 0);
    }

    #[test]
    fn test_mixed_input() {
        let lexemes = lex_input("(atom \"string\" ' ` ,)");
        assert_eq!(lexemes.len(), 7);
        assert!(matches!(lexemes[0].token, Token::LParen));
        assert_eq!(lexemes[0].position, 0);
        assert!(matches!(lexemes[1].token, Token::Identifier(ref s) if s == "atom"));
        assert_eq!(lexemes[1].position, 1);
        assert!(matches!(lexemes[2].token, Token::String(ref s) if s == "string"));
        assert_eq!(lexemes[2].position, 6);
        assert!(matches!(lexemes[3].token, Token::Quote));
        assert_eq!(lexemes[3].position, 15);
        assert!(matches!(lexemes[4].token, Token::Backquote));
        assert_eq!(lexemes[4].position, 17);
        assert!(matches!(lexemes[5].token, Token::Comma));
        assert_eq!(lexemes[5].position, 19);
        assert!(matches!(lexemes[6].token, Token::RParen));
        assert_eq!(lexemes[6].position, 20);
    }

    #[test]
    fn test_comment() {
        let lexemes = lex_input("; this is a comment\natom");
        assert_eq!(lexemes.len(), 1);
        assert!(matches!(lexemes[0].token, Token::Identifier(ref s) if s == "atom"));
        assert_eq!(lexemes[0].position, 20);
    }

    #[test]
    fn test_string_with_escape() {
        let lexemes = lex_input("\"hello \\\"world\\\"\"");
        assert_eq!(lexemes.len(), 1);
        assert!(matches!(lexemes[0].token, Token::String(ref s) if s == "hello \\\"world\\\""));
        assert_eq!(lexemes[0].position, 0);
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
        assert!(matches!(lexemes[0].token, Token::Identifier(ref s) if s == "atom"));
        assert_eq!(lexemes[0].position, 0);
        assert!(matches!(lexemes[1].token, Token::LParen));
        assert_eq!(lexemes[1].position, 4);
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
    fn test_lexeme_positions_01() {
        let input = "0 23 5678";
        let lexemes = lex_input(input);
        assert_eq!(lexemes.len(), 3);

        let expected_positions = vec![0, 2, 5];
        for (lexeme, &expected_position) in lexemes.iter().zip(expected_positions.iter()) {
            assert_eq!(lexeme.position, expected_position, "Lexeme: {:?}", lexeme);
        }
    }

    #[test]
    fn test_lexeme_positions_02() {
        let input = "((((((";
        let lexemes = lex_input(input);
        assert_eq!(lexemes.len(), 6);

        let expected_positions = vec![0, 1, 2, 3, 4, 5];
        for (lexeme, &expected_position) in lexemes.iter().zip(expected_positions.iter()) {
            assert_eq!(lexeme.position, expected_position, "Lexeme: {:?}", lexeme);
        }
    }

    #[test]
    fn test_lexeme_positions_03() {
        let input = "(atom)";
        let lexemes = lex_input(input);
        assert_eq!(lexemes.len(), 3);

        let expected_positions = vec![0, 1, 5];
        for (lexeme, &expected_position) in lexemes.iter().zip(expected_positions.iter()) {
            assert_eq!(lexeme.position, expected_position, "Lexeme: {:?}", lexeme);
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
            if let Token::Identifier(ref s) = lexeme.token {
                assert!(!s.contains(' '), "Atom token contains whitespace: {}", s);
            }
        }
    }

    #[test]
    fn test_lexeme_line_and_column_01() {
        let input = "atom\nsecond";
        let lexemes = lex_input(input);
        assert_eq!(lexemes.len(), 2, "Lexemes: {:?}", lexemes);

        assert_eq!(lexemes[0].line, 1);
        assert_eq!(lexemes[0].column, 1);
        assert_eq!(lexemes[1].line, 2, "Lexemes: {:#?}", lexemes);
        assert_eq!(lexemes[1].column, 1, "Lexeme: {:?}", lexemes[1]);
    }

    #[test]
    fn test_lexeme_line_and_column_02() {
        let input = "(print \"Hello, World!\")\n\n(print \"Another line\")";
        let lexemes = lex_input(input);
        assert_eq!(lexemes.len(), 8);

        assert_eq!(lexemes[0].line, 1); // (
        assert_eq!(lexemes[0].column, 1);
        assert_eq!(lexemes[1].line, 1); // print
        assert_eq!(lexemes[1].column, 2);
        assert_eq!(lexemes[2].line, 1); // Hello, World!
        assert_eq!(lexemes[2].column, 8);
        assert_eq!(lexemes[3].line, 1); // )
        assert_eq!(lexemes[3].column, 23);
        assert_eq!(lexemes[4].line, 3); // (
        assert_eq!(lexemes[4].column, 1);
        assert_eq!(lexemes[5].line, 3); // print
        assert_eq!(lexemes[5].column, 2);
        assert_eq!(lexemes[6].line, 3); // Another line
        assert_eq!(lexemes[6].column, 8);
        assert_eq!(lexemes[7].line, 3); // )
        assert_eq!(lexemes[7].column, 22);
    }

    #[test]
    fn test_integer() {
        let lexemes = lex_input("12345");
        assert_eq!(lexemes.len(), 1);
        assert!(
            matches!(lexemes[0].token, Token::Integer(ref s) if s == "12345"),
            "{:?}",
            lexemes[0].token
        );
        assert_eq!(lexemes[0].position, 0);
    }

    #[test]
    fn test_negative_integer() {
        let lexemes = lex_input("-12345");
        assert_eq!(lexemes.len(), 1);
        assert!(matches!(lexemes[0].token, Token::Integer(ref s) if s == "-12345"));
        assert_eq!(lexemes[0].position, 0);
    }

    #[test]
    fn test_float() {
        let lexemes = lex_input("123.45");
        assert_eq!(lexemes.len(), 1);
        assert!(matches!(lexemes[0].token, Token::Float(ref s) if s == "123.45"));
        assert_eq!(lexemes[0].position, 0);
    }

    #[test]
    fn test_negative_float() {
        let lexemes = lex_input("-123.45");
        assert_eq!(lexemes.len(), 1);
        assert!(matches!(lexemes[0].token, Token::Float(ref s) if s == "-123.45"));
        assert_eq!(lexemes[0].position, 0);
    }

    #[test]
    fn test_float_with_exponent() {
        let lexemes = lex_input("1.23e10");
        assert_eq!(lexemes.len(), 1);
        assert!(matches!(lexemes[0].token, Token::Float(ref s) if s == "1.23e10"));
        assert_eq!(lexemes[0].position, 0);
    }

    #[test]
    fn test_negative_float_with_exponent() {
        let lexemes = lex_input("-1.23e-10");
        assert_eq!(lexemes.len(), 1, "{:?}", lexemes);
        assert!(matches!(lexemes[0].token, Token::Float(ref s) if s == "-1.23e-10"));
        assert_eq!(lexemes[0].position, 0);
    }

    #[test]
    fn test_integer_followed_by_float() {
        let lexemes = lex_input("123 456.78");
        assert_eq!(lexemes.len(), 2);
        assert!(matches!(lexemes[0].token, Token::Integer(ref s) if s == "123"));
        assert_eq!(lexemes[0].position, 0);
        assert!(matches!(lexemes[1].token, Token::Float(ref s) if s == "456.78"));
        assert_eq!(lexemes[1].position, 4);
    }

    #[test]
    fn test_float_followed_by_integer() {
        let lexemes = lex_input("123.45 678");
        assert_eq!(lexemes.len(), 2);
        assert!(matches!(lexemes[0].token, Token::Float(ref s) if s == "123.45"));
        assert_eq!(lexemes[0].position, 0);
        assert!(matches!(lexemes[1].token, Token::Integer(ref s) if s == "678"));
        assert_eq!(lexemes[1].position, 7);
    }
}
