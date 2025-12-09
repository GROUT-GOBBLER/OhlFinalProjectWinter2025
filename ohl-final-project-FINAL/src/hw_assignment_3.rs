#![allow(non_camel_case_types)]
#![allow(dead_code)]


use std::fs;
use std::mem::discriminant;

#[derive(Debug, Clone)]
pub enum Token {
    PARENS_L, PARENS_R,
    BRACKET_L, BRACKET_R,
    BRACE_L, BRACE_R,
    POINT, COMMA, COLON, SEMICOLON, ARROW_R,
    // arithmetic operators
    ADD, SUB, MUL, DIV,
    ADD_ASSIGN, SUB_ASSIGN, MUL_ASSIGN, DIV_ASSIGN,
    // relational operators
    EQ,     // equal
    LT,     // less than
    GT,     // greater than
    NEQ,    // not equal
    NLT,    // not less than == greater than or equal
    NGT,    // not greater than == less than or equal
    // logical operators
    NOT, AND, OR,
    // other operators
    ASSIGN, AMPERSAND,
    // keywords
    MUT, FUNC, LET, IF, ELSE, ELSE_IF, WHILE, PRINT, RETURN,

    // types
    TYPE_INT32, TYPE_FLT32, TYPE_CHAR,

    // literals
    ID(String),
    LIT_INT32(i32),
    LIT_FLT32(f32),
    LIT_CHAR(char),
    LIT_STRING(String),
    TRUE,
    FALSE,


    // special characters
    LINEBREAK, EOI, ERROR,

    // Meta tokens for AST
    META_PROGRAM, META_FUNC, META_PARAM_LIST, META_PARAM,
    META_LET, META_RETURN, META_IF, META_ELSE_IF,
    META_BLOCK, META_VOID, META_INFER,
    META_ASSIGN, META_CALL, META_PRINT, META_WHILE,

    // Keywords.
    READ, WRITE
}

impl Eq for Token { }

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        discriminant(self) == discriminant(other)
    }
}

impl Token {
    pub fn id() -> Token {
        Token::ID(String::new())
    }
    pub fn lit_i32() -> Token { Token::LIT_INT32(0) }
    pub fn lit_f32() -> Token { Token::LIT_FLT32(0.0) }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LexerState {
    Start,
    Operation,
    Stage,
    NumberInt,
    NumberFloat,
    NumberChar,
    NumberString,
    NumberArray,
    CharArray,
    StringArray,
    CharLit,
    StringLit,
    Return
}

#[derive(Clone)]
pub struct Lexer {
    pub input_string: String,
    pub input_pos: usize,
    pub state: LexerState,
    pub token: Option<Token>,
    pub buffer_string: String,
    pub tokens : Vec<Token>,
}

impl Lexer {
    pub fn new() -> Self {
        Lexer {
            input_string: String::new(),
            input_pos: 0,
            state: LexerState::Start,
            token: None,
            buffer_string: String::new(),
            tokens : vec![],
        }
    }

    pub fn set_input(&mut self, input: String) {
        let input_string = fs::read_to_string(input).expect("Should have been able to read the input file");
        self.input_string = input_string;
        self.input_pos = 0;
        self.state = LexerState::Start;
        self.token = None;
        self.buffer_string.clear();
    }

    pub fn get_next_character_is_value(&mut self, val: char) -> bool {
        return self.input_pos + 1 < self.input_string.len() && self.input_string.as_bytes()[self.input_pos + 1] as char == val;
    }

    fn next_char(&self) -> Option<char> {
        self.input_string.as_bytes().get(self.input_pos).map(|&b| b as char)
    }

    fn peek_char(&self) -> Option<char> {
        self.input_string.as_bytes().get(self.input_pos + 1).map(|&b| b as char)
    }
    /*
    pub fn skip_whitespace(&mut self) {
        while self.input_pos < self.input_string.len() {
            let ch = self.next_char().unwrap();
            if ch.is_whitespace() && ch != '\n' {
                self.input_pos += 1;
            } else {
                break;
            }
        }
    }
    */
    pub fn advance(&mut self) -> Option<Token> {
        self.token = None;

        while self.input_pos < self.input_string.len() {
            let current_char: char = self.input_string.as_bytes()[self.input_pos] as char;
            // let current_char: char = self.next_char().unwrap();
            match self.state {
                LexerState::Start => {
                    if vec!['\n'].contains(&current_char) {
                        self.input_pos += 1;
                        let token = Token::LINEBREAK;
                        self.token = Some(token.clone());
                        return Some(token);
                    }
                    if current_char.is_whitespace() {
                        self.input_pos += 1;
                        continue;
                    }

                    if self.input_pos >= self.input_string.len() {
                        let token = Token::EOI;
                        self.token = Some(token.clone());
                        return Some(token);
                    }
                    self.buffer_string.clear();
                    if current_char.is_alphabetic() || current_char == '_' || current_char.is_digit(10) {
                        self.input_pos += 1;
                        self.buffer_string.push(current_char);
                        self.state = if current_char.is_digit(10) { LexerState::NumberInt } else { LexerState::Operation };
                        continue;
                    }

                    if current_char == '\'' {
                        self.input_pos += 1;
                        self.state = LexerState::CharLit;
                        continue;
                    }
                    if current_char == '"' {
                        self.input_pos += 1;
                        self.state = LexerState::StringLit;
                        continue;
                    }
                    if vec!['(', ')', '[', ']', '{', '}', ',', ':', ';', '&', '|'].contains(&current_char) {
                        let token: Token;
                        match current_char {
                            '(' => token = Token::PARENS_L,
                            ')' => token = Token::PARENS_R,
                            '[' => token = Token::BRACKET_L,
                            ']' => token = Token::BRACKET_R,
                            '{' => token = Token::BRACE_L,
                            '}' => token = Token::BRACE_R,
                            ',' => token = Token::COMMA,
                            ':' => token = Token::COLON,
                            ';' => token = Token::SEMICOLON,
                            '&' => {
                                if self.peek_char() == Some('&') {
                                    self.input_pos += 1;
                                    token = Token::AND;
                                } else {
                                    token = Token::AMPERSAND;
                                }
                            },
                            '|' => {
                                if self.peek_char() == Some('|') {
                                    self.input_pos += 1;
                                    token = Token::OR;
                                } else {
                                    token = Token::ERROR;
                                }
                            }
                            _ => token = Token::ID(current_char.to_string()),
                        }
                        self.input_pos += 1;
                        self.token = Some(token.clone());
                        return Some(token)
                    }

                    if current_char == '+' {
                        let peek = self.peek_char();
                        let token = if peek == Some('=') {
                            self.input_pos += 1;
                            Token::ADD_ASSIGN
                        } else {
                            Token::ADD
                        };
                        self.input_pos += 1;
                        self.token = Some(token.clone());
                        return Some(token);
                    }
                    if current_char == '*' {
                        let peek = self.peek_char();
                        let token = if peek == Some('=') {
                            self.input_pos += 1;
                            Token::MUL_ASSIGN
                        } else {
                            Token::MUL
                        };
                        self.input_pos += 1;
                        self.token = Some(token.clone());
                        return Some(token);
                    }
                    if current_char == '/' {
                        let peek = self.peek_char();
                        let token = if peek == Some('=') {
                            self.input_pos += 1;
                            Token::DIV_ASSIGN
                        } else {
                            Token::DIV
                        };
                        self.input_pos += 1;
                        self.token = Some(token.clone());
                        return Some(token);
                    }

                    if current_char == '.' {
                        self.input_pos += 1;
                        if self.input_pos < self.input_string.len() && (self.input_string.as_bytes()[self.input_pos] as char).is_digit(10) {
                            self.buffer_string.push('.');
                            self.state = LexerState::NumberFloat;
                            continue;
                        } else {
                            let token = Token::POINT;
                            self.token = Some(token.clone());
                            return Some(token);
                        }
                    }
                    if vec!['=', '<', '>', '!'].contains(&current_char) {
                        let token: Token;
                        let has_found_equals: bool = self.peek_char() == Some('=');
                        match current_char {
                            '=' => token = if has_found_equals { Token::EQ } else { Token::ASSIGN },
                            '<' => token = if has_found_equals { Token::NGT } else { Token::LT },
                            '>' => token = if has_found_equals { Token::NLT } else { Token::GT },
                            '!' => {
                                if self.input_pos + 1 < self.input_string.len() {
                                    token = if has_found_equals {Token::NEQ } else { Token::NOT }
                                } else {
                                    panic!("Invalid character: !");
                                }
                            },
                            _ => {token = Token::ID(current_char.to_string())}
                        }
                        self.input_pos += if has_found_equals { 2 } else { 1 };
                        self.token = Some(token.clone());
                        return Some(token);
                    }
                    if current_char == '-' {
                        let next = self.peek_char();
                        let token = if next == Some('>') {
                            Token::ARROW_R
                        } else if next == Some('=') {
                            Token::SUB_ASSIGN
                        } else {
                            Token::SUB
                        };
                        self.input_pos += if next.is_some() { 2 } else { 1 };
                        self.token = Some(token.clone());
                        return Some(token);
                    }
                }
                LexerState::Operation => {
                    if current_char.is_alphanumeric() || current_char == '_' {
                        self.buffer_string.push(current_char);
                        self.input_pos += 1;
                        continue;
                    } else {
                        let token = match self.buffer_string.as_str() {
                            "func" => Token::FUNC,
                            "let" => Token::LET,
                            "if" => Token::IF,
                            "else" => Token::ELSE,
                            "else if" => Token::ELSE_IF,
                            "while" => Token::WHILE,
                            "print" => Token::PRINT,
                            "return" => Token::RETURN,
                            "not" => Token::NOT,
                            "and" => Token::AND,
                            "or" => Token::OR,
                            "int32" => Token::TYPE_INT32,
                            "flt32" => Token::TYPE_FLT32,
                            "char" => Token::TYPE_CHAR,
                            "mut" => Token::MUT,
                            "true" => Token::TRUE,
                            "false" => Token::FALSE,
                            _ => Token::ID(self.buffer_string.clone()),
                        };
                        self.state = LexerState::Start;
                        self.buffer_string.clear();
                        self.token = Some(token.clone());
                        return Some(token);
                    }
                }
                LexerState::NumberInt => {
                    if current_char.is_digit(10) || current_char == '.' {
                        self.buffer_string.push(current_char);
                        self.input_pos += 1;
                        if current_char == '.' {
                            self.state = LexerState::NumberFloat;
                        }
                        continue;
                    } else {
                        let val = self.buffer_string.parse::<i32>().expect("Invalid value for integer");
                        self.buffer_string.clear();
                        self.state = LexerState::Start;
                        let token = Token::LIT_INT32(val);
                        self.token = Some(token.clone());
                        return Some(token);
                    }
                }
                LexerState::NumberFloat => {
                    if current_char.is_digit(10) {
                        self.buffer_string.push(current_char);
                        self.input_pos += 1;
                        continue;
                    } else {
                        let val = self.buffer_string.parse::<f32>().expect("Invalid value for float");
                        self.buffer_string.clear();
                        self.state = LexerState::Start;
                        let token = Token::LIT_FLT32(val);
                        self.token = Some(token.clone());
                        return Some(token);
                    }
                }
                LexerState::CharLit => {
                    self.input_pos += 1;
                    if self.next_char() != Some('\'') {
                        panic!("Char literal {} not closed", '\'');
                    }
                    self.input_pos += 1;
                    self.state = LexerState::Start;
                    let token = Token::LIT_CHAR(current_char);
                    self.token = Some(token.clone());
                    return Some(token);

                }
                LexerState::StringLit => {
                    if current_char == '"' {
                        let buffer_clone = self.buffer_string.clone();
                        self.buffer_string.clear();
                        self.input_pos += 1;
                        self.state = LexerState::Start;
                        let token = Token::LIT_STRING(buffer_clone);
                        self.token = Some(token.clone());
                        return Some(token);
                    } else {
                        self.buffer_string.push(current_char);
                        self.input_pos += 1;
                        continue;
                    }
                }
                _ => {}
            }
        }
        let token = Token::EOI;
        self.token = Some(token.clone());
        Some(token)
    }

    pub fn curr(&self) -> Token {
        self.token.clone().unwrap_or(Token::EOI)
    }

    pub fn print_tokens(&mut self) {
        self.advance();
        while true {
            self.tokens.push(self.curr());
            let token = self.curr();
            match &token {
                Token::PARENS_L => print!("PARENS_L, "),
                Token::PARENS_R => print!("PARENS_R, "),
                Token::BRACKET_L => print!("BRACKET_L, "),
                Token::BRACKET_R => print!("BRACKET_R, "),
                Token::BRACE_L => print!("BRACE_L, "),
                Token::BRACE_R => print!("BRACE_R, "),
                Token::POINT => print!("POINT, "),
                Token::COMMA => print!("COMMA, "),
                Token::COLON => print!("COLON, "),
                Token::SEMICOLON => print!("SEMICOLON, "),
                Token::ARROW_R => print!("RIGHTARROW, "),
                Token::ADD => print!("ADD, "),
                Token::SUB => print!("SUB, "),
                Token::MUL => print!("MUL, "),
                Token::DIV => print!("DIV, "),
                Token::ADD_ASSIGN => print!("ADD_ASSIGN, "),
                Token::SUB_ASSIGN => print!("SUB_ASSIGN, "),
                Token::MUL_ASSIGN => print!("MUL_ASSIGN, "),
                Token::DIV_ASSIGN => print!("DIV_ASSIGN, "),
                Token::EQ => print!("EQ, "),
                Token::LT => print!("LT, "),
                Token::GT => print!("GT, "),
                Token::NEQ => print!("NEQ, "),
                Token::NLT => print!("NLT, "),
                Token::NGT => print!("NGT, "),
                Token::NOT => print!("NOT, "),
                Token::AND => print!("AND, "),
                Token::OR => print!("OR, "),
                Token::ASSIGN => print!("ASSIGN, "),
                Token::AMPERSAND => print!("AMPERSAND, "),
                Token::MUT => print!("MUT, "),
                Token::FUNC => print!("FUNC, "),
                Token::LET => print!("LET, "),
                Token::IF => print!("IF, "),
                Token::ELSE => print!("ELSE, "),
                Token::ELSE_IF => print!("ELSE_IF, "),
                Token::WHILE => print!("WHILE, "),
                Token::PRINT => print!("PRINT, "),
                Token::RETURN => print!("RETURN, "),
                Token::ID(token_string) => print!("ID(\"{}\"), ", token_string),
                Token::LINEBREAK => println!(),
                Token::TYPE_INT32 => print!("TYPE_INT32, "),
                Token::TYPE_FLT32 => print!("TYPE_FLT32, "),
                Token::TYPE_CHAR => print!("TYPE_CHAR, "),
                Token::LIT_INT32(token_int32) => print!("LIT_INT32({}), ", token_int32),
                Token::LIT_FLT32(token_f32) => print!("LIT_FLT32({}), ", token_f32),
                Token::LIT_CHAR(token_char) => print!("LIT_CHAR('{}'), ", token_char),
                Token::LIT_STRING(token_string) => print!("LIT_STRING(\"{}\"), ", token_string),
                Token::TRUE => print!("TRUE, "),
                Token::FALSE => print!("FALSE, "),
                Token::ERROR => print!("ERROR, "),
                Token::META_PROGRAM => print!("META_PROGRAM, "),
                Token::META_FUNC => print!("META_FUNC, "),
                Token::META_PARAM_LIST => print!("META_PARAM_LIST, "),
                Token::META_PARAM => print!("META_PARAM, "),
                Token::META_LET => print!("META_LET, "),
                Token::META_RETURN  => print!("META_RETURN, "),
                Token::META_IF => print!("META_IF, "),
                Token::META_ELSE_IF => print!("META_ELSE_IF, "),
                Token::META_BLOCK => print!("META_BLOCK, "),
                Token::META_VOID => print!("META_VOID, "),
                Token::META_INFER => print!("META_INFER, "),
                Token::META_ASSIGN => print!("META_ASSIGN, "),
                Token::META_CALL => print!("META_CALL, "),
                Token::META_PRINT => print!("META_PRINT, "),
                Token::META_WHILE => print!("META_WHILE, "),
                Token::EOI => {
                    println!("\n\nEOI");
                    break;
                }
                _ => print!("{:?}, ", token),
            }
            self.advance();
        }
    }
}
