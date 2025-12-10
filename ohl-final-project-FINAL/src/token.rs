#![allow(non_camel_case_types)]
#![allow(dead_code)]

use std::cell::RefCell;
use std::fmt;
use std::mem::discriminant;
use std::rc::Rc;
use crate::frame_analyze::{AFrame, CellLoc};
use crate::value::DValue;

#[derive(Clone)]
pub enum TCode {
    // general
    EOI,
    ERROR,

    // id, typ, value atoms
    ID(String),
    VAL(DValue),

    // assignment operator
    ASSIGN,

    // logical operators
    NOT,
    AND,
    OR,

    // relational operators
    LT,          // less than
    GT,          // greater than
    EQ,       // equal
    NOT_EQ,   // not equal

    // arithmetic operators
    ADD,
    SUB,
    MULT,
    DIV,

    // nesting
    PAREN_L,
    PAREN_R,
    BRACE_L,
    BRACE_R,

    // separators
    COMMA,
    SEMICOLON,

    // keywords
    FUNC,
    LET,
    IF,
    ELSE,
    WHILE,
    RETURN,
    READ,
    WRITE,

    // meta tokens
    BLOCK,
    // FUNC,
    PARAMS,
    CALL,

    A_BLOCK(Rc<RefCell<AFrame>>),
    A_REF(CellLoc)
}

impl fmt::Debug for TCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            TCode::EOI => write!(f, "EOI"),
            TCode::ERROR => write!(f, "ERROR"),
            TCode::ID(name) => write!(f, "ID(\"{}\")", name),
            TCode::VAL(value) => write!(f, "VAL({:?})", value),
            TCode::ASSIGN => write!(f, ":="),
            TCode::OR => write!(f, "OR"),
            TCode::AND => write!(f, "AND"),
            TCode::NOT => write!(f, "NOT"),
            TCode::LT => write!(f, "<"),
            TCode::GT => write!(f, ">"),
            TCode::EQ => write!(f, "=="),
            TCode::NOT_EQ => write!(f, "!="),
            TCode::ADD => write!(f, "+"),
            TCode::SUB => write!(f, "-"),
            TCode::MULT => write!(f, "*"),
            TCode::DIV => write!(f, "/"),
            TCode::PAREN_L => write!(f, "("),
            TCode::PAREN_R => write!(f, ")"),
            TCode::BRACE_L => write!(f, "["),
            TCode::BRACE_R => write!(f, "]"),
            TCode::COMMA => write!(f, ","),
            TCode::SEMICOLON => write!(f, ";"),
            TCode::LET => write!(f, "LET"),
            TCode::IF => write!(f, "ID"),
            TCode::ELSE => write!(f, "ELSE"),
            TCode::WHILE => write!(f, "WHILE"),
            TCode::RETURN => write!(f, "RET"),
            TCode::READ => write!(f, "READ"),
            TCode::WRITE => write!(f, "WRITE"),
            TCode::PARAMS => write!(f, "PARAMS"),
            TCode::BLOCK => write!(f, "BLOCK"),
            TCode::A_BLOCK(rcc_frame) => {
                write!(f, "A_BLOCK").expect("");
                let s = rcc_frame.borrow().print();
                write!(f, "{:}", s.as_str())
            },
            TCode::CALL => write!(f, "CALL"),
            TCode::FUNC => write!(f, "FUNC"),
            TCode::A_REF(loc) => write!(f, "REF {:?}", loc),
        }
    }
}

/// token position
#[derive(Debug, Clone)]
pub struct TPos {
    pub row: usize,     // row in source code
    pub col: usize,     // column in source code
    pub len: usize,     // number of characters in source code
}

impl TPos {
    fn new(row: usize, col: usize, len: usize) -> TPos {
        TPos { row, col, len }
    }
}

#[derive(Debug, Clone)]
pub struct TLoc {
    pub first: TPos,    // position and length of token
    pub last: TPos,     // position and length of corresponding delimiter
}

impl TLoc {
    pub fn empty() -> TLoc {
        TLoc {
            first: TPos::new(0, 0, 0),
            last: TPos::new(0, 0, 0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub code: TCode,    // code of token (such as EOI)
    pub loc: TLoc       // location of token
}

impl PartialEq for TCode {
    fn eq(&self, other: &Self) -> bool {
        discriminant(self) == discriminant(other)
    }
}

impl Eq for TCode { }

impl TCode {
    pub fn eoi() -> TCode { TCode::EOI }

    pub fn id() -> TCode {
        TCode::ID(String::new())
    }
}



impl Token {

    pub fn from(code : TCode) -> Token {
        Token { code, loc: TLoc::empty() }
    }

    pub fn error(_reason : &str) -> Token {
        Token {
            code: TCode::ERROR,
            loc: TLoc::empty()
        }
    }

    pub fn id(name: &str) -> Token {
        Token {
            code: TCode::ID(String::from(name)),
            loc: TLoc::empty()
        }
    }


}