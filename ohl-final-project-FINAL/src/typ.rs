#![allow(dead_code)]

/// atomic type
#[derive(Debug, Copy, Clone)]
pub enum ATyp {
    TOK,
    BOOL,
    CHAR,
    I64,
    F64,
}

/// composite type
#[derive(Debug, Clone)]
pub enum CTyp {
    FUNC(Box<Typ>,Box<Typ>),
    TUPLE(Vec<Typ>),
    LIST(Box<Typ>),
    MAP(Box<Typ>, Box<Typ>),
}

/// type (either atomic, composite, or dynamic)
#[derive(Debug, Clone)]
pub enum Typ {
    A(ATyp),
    C(CTyp),
    D
}

impl ATyp {

    pub fn isNumeric(&self) -> bool {
        match self {
            ATyp::I64 => { true }
            ATyp::F64 => { true }
            _ => { false }
        }
    }

    pub(crate) fn getCommon(&self, other: ATyp) -> ATyp {
        match self {
            ATyp::CHAR => {
                match other {
                    ATyp::CHAR => { ATyp::CHAR }
                    ATyp::I64 => { ATyp::I64 }
                    ATyp::F64 => { ATyp::F64 }
                    _ => { ATyp::TOK }
                }
            }
            ATyp::I64 => {
                match other {
                    ATyp::CHAR => { ATyp::I64 }
                    ATyp::I64 => { ATyp::I64 }
                    ATyp::F64 => { ATyp::F64 }
                    _ => { ATyp::TOK }
                }
            }
            ATyp::F64 => {
                match other {
                    ATyp::CHAR => { ATyp::F64 }
                    ATyp::I64 => { ATyp::F64 }
                    ATyp::F64 => { ATyp::F64 }
                    _ => { ATyp::TOK }
                }
            }
            ATyp::BOOL => {
                match other {
                    ATyp::BOOL => { ATyp::BOOL }
                    _ => { ATyp::TOK }
                }
            }
            _ => { ATyp::TOK }
        }
    }

}

impl Typ {

    pub fn isNumeric(&self) -> bool {
        match self {
            Typ::A(a) => {
                a.isNumeric()
            }
            _ => { false }
        }
    }

    pub fn isFunc(&self) -> bool {
        match self {
            Typ::C(CTyp::FUNC(_,_)) => { true }
            _ => { false }
        }
    }

}