#![allow(non_camel_case_types)]
#![allow(dead_code)]

use std::cell::RefCell;
use std::fmt;
use std::mem::discriminant;
use std::rc::Rc;
use crate::frame_analyze::{AFrame, CellLoc};
use crate::typ::{ATyp};
use crate::value::DValue;

#[derive(Clone)]
pub enum TCode {

    // general
    EOI,
    ERROR,

    // id, typ, value atoms
    ID(String),
    TYP(ATyp),
    VAL(DValue),

    // assignment operator
    OP_ASSIGN,

    // logical operators
    OP_OR,
    OP_AND,
    OP_NOT,

    // relational operators
    OP_LT,          // less than
    OP_GT,          // greater than
    OP_NOT_LT,      // not less than == greater than or equal
    OP_NOT_GT,      // not greater than == less than or equal
    OP_EQUAL,       // equal
    OP_NOT_EQUAL,   // not equal

    // arithmetic operators
    OP_ADD,
    OP_SUB,
    OP_MUL,
    OP_DIV,
    OP_POW,

    // nesting
    PAREN_L,
    PAREN_R,
    BRACKET_L,
    BRACKET_R,
    BRACE_L,
    BRACE_R,

    // separators
    POINT,
    COMMA,
    COLON,
    SEMICOLON,
    ARROW_R,

    // keywords
    // KW_FUNC,
    KW_LET,
    KW_IF,
    KW_ELSE,
    KW_WHILE,
    KW_RETURN,
    KW_READ,
    KW_WRITE,

    // meta tokens
    BLOCK,
    FUNC,
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
            TCode::TYP(_) => write!(f, "TYP"),
            TCode::OP_ASSIGN => write!(f, ":="),
            TCode::OP_OR => write!(f, "OR"),
            TCode::OP_AND => write!(f, "AND"),
            TCode::OP_NOT => write!(f, "NOT"),
            TCode::OP_LT => write!(f, "<"),
            TCode::OP_GT => write!(f, ">"),
            TCode::OP_NOT_LT => write!(f, ">="),
            TCode::OP_NOT_GT => write!(f, "<="),
            TCode::OP_EQUAL => write!(f, "=="),
            TCode::OP_NOT_EQUAL => write!(f, "!="),
            TCode::OP_ADD => write!(f, "+"),
            TCode::OP_SUB => write!(f, "-"),
            TCode::OP_MUL => write!(f, "*"),
            TCode::OP_DIV => write!(f, "/"),
            TCode::OP_POW => write!(f, "^"),
            TCode::PAREN_L => write!(f, "("),
            TCode::PAREN_R => write!(f, ")"),
            TCode::BRACKET_L => write!(f, "["),
            TCode::BRACKET_R => write!(f, "]"),
            TCode::BRACE_L => write!(f, "{{"),
            TCode::BRACE_R => write!(f, "}}"),
            TCode::POINT => write!(f, "."),
            TCode::COMMA => write!(f, ","),
            TCode::COLON => write!(f, ":"),
            TCode::SEMICOLON => write!(f, ";"),
            TCode::ARROW_R => write!(f, "->"),
            TCode::KW_LET => write!(f, "LET"),
            TCode::KW_IF => write!(f, "ID"),
            TCode::KW_ELSE => write!(f, "ELSE"),
            TCode::KW_WHILE => write!(f, "WHILE"),
            TCode::KW_RETURN => write!(f, "RET"),
            TCode::KW_READ => write!(f, "READ"),
            TCode::KW_WRITE => write!(f, "WRITE"),
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