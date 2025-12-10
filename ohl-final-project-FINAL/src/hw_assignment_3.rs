#![allow(non_camel_case_types)]
#![allow(dead_code)]

use std::fs;
use std::mem::discriminant;
use crate::token::{TCode};
use crate::value::DValue;

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
    pub token: Option<TCode>,
    pub buffer_string: String,
    pub tokens : Vec<TCode>,
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
    pub fn advance(&mut self) -> Option<TCode> {
        self.token = None;

        while self.input_pos < self.input_string.len() {
            let current_char: char = self.input_string.as_bytes()[self.input_pos] as char;
            // let current_char: char = self.next_char().unwrap();
            match self.state {
                LexerState::Start => {
                    if vec!['\n'].contains(&current_char) {
                        self.input_pos += 1;
                        println!();
                        continue;
                    }
                    if current_char.is_whitespace() {
                        self.input_pos += 1;
                        continue;
                    }

                    if self.input_pos >= self.input_string.len() {
                        let token = TCode::EOI;
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
                        let token: TCode;
                        match current_char {
                            '(' => token = TCode::PAREN_L,
                            ')' => token = TCode::PAREN_R,
                            ']' => token = TCode::BRACE_R,
                            '[' => token = TCode::BRACE_L,
                            ',' => token = TCode::COMMA,
                            //':' => token = TCode::COLON,
                            ';' => token = TCode::SEMICOLON,
                            '&' => {
                                if self.peek_char() == Some('&') {
                                    self.input_pos += 1;
                                    token = TCode::AND;
                                } else {
                                    token = TCode::ERROR;
                                }
                            },
                            '|' => {
                                if self.peek_char() == Some('|') {
                                    self.input_pos += 1;
                                    token = TCode::OR;
                                } else {
                                    token = TCode::ERROR;
                                }
                            }
                            _ => token = TCode::ID(current_char.to_string()),
                        }
                        self.input_pos += 1;
                        self.token = Some(token.clone());
                        return Some(token)
                    }

                    if current_char == '+' {
                        let peek = self.peek_char();
                        let token = TCode::ADD;
                        self.input_pos += 1;
                        self.token = Some(token.clone());
                        return Some(token);
                    }
                    if current_char == '*' {
                        let peek = self.peek_char();
                        let token = TCode::MULT;
                        self.input_pos += 1;
                        self.token = Some(token.clone());
                        return Some(token);
                    }
                    if current_char == '/' {
                        let peek = self.peek_char();
                        let token = TCode::DIV;
                        self.input_pos += 1;
                        self.token = Some(token.clone());
                        return Some(token);
                    }

                    if vec!['=', '<', '>', '!'].contains(&current_char) {
                        let token: TCode;
                        let has_found_equals: bool = self.peek_char() == Some('=');
                        match current_char {
                            '=' => token = if has_found_equals { TCode::EQ } else { TCode::ASSIGN },
                            '<' => token = TCode::LT,
                            '>' => token = TCode::GT,
                            '!' => {
                                if self.input_pos + 1 < self.input_string.len() {
                                    token = if has_found_equals {TCode::NOT_EQ } else { TCode::NOT }
                                } else {
                                    panic!("Invalid character: !");
                                }
                            },
                            _ => {token = TCode::ID(current_char.to_string())}
                        }
                        self.input_pos += if has_found_equals { 2 } else { 1 };
                        self.token = Some(token.clone());
                        return Some(token);
                    }
                    if current_char == '-' {
                        let next = self.peek_char();
                        let token = TCode::SUB;
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
                            "true" => TCode::VAL(DValue::BOOL(true)),
                            "false" => TCode::VAL(DValue::BOOL(false)),
                            "func" => TCode::FUNC,
                            "let" => TCode::LET,
                            "if" => TCode::IF,
                            "else" => TCode::ELSE,
                            "while" => TCode::WHILE,
                            //"print" => TCode::PRINT,
                            "return" => TCode::RETURN,
                            "not" => TCode::NOT,
                            "and" => TCode::AND,
                            "or" => TCode::OR,
                            _ => TCode::ID(self.buffer_string.clone()),
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
                        let token = TCode::VAL(DValue::I64(val as i64));
                        self.token = Some(token.clone());
                        return Some(token);
                    }
                }
                /*LexerState::StringLit => {
                    if current_char == '"' {
                        let buffer_clone = self.buffer_string.clone();
                        self.buffer_string.clear();
                        self.input_pos += 1;
                        self.state = LexerState::Start;
                        let token = TCode::LIT_STRING(buffer_clone);
                        self.token = Some(token.clone());
                        return Some(token);
                    } else {
                        self.buffer_string.push(current_char);
                        self.input_pos += 1;
                        continue;
                    }
                }*/
                _ => {}
            }
        }
        let token = TCode::EOI;
        self.token = Some(token.clone());
        Some(token)
    }

    pub fn curr(&self) -> TCode {
        self.token.clone().unwrap_or(TCode::EOI)
    }

    pub fn print_token(&mut self) {
        let token = self.token.clone().unwrap();

        match token {
            TCode::EOI => {
                println!("\nEOI");
            },

            _ => {
                print!("{:?} ", token);
            }
        }
    }
}