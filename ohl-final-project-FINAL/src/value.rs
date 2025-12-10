use std::collections::HashMap;
use std::rc::Rc;
use crate::mtree::MTree;
use crate::token::TCode;
use crate::typ::{ATyp, CTyp, Typ};

impl TCode {

    pub fn isLogicalOp(&self) -> bool {
        match self {
            TCode::OR => { true }
            TCode::AND => { true }
            TCode::NOT => { true }
            _ => { false }
        }
    }

    pub fn isRelationalOp(&self) -> bool {
        match self {
            TCode::LT => { true }
            TCode::GT => { true }
            TCode::EQ => { true }
            TCode::NOT_EQ => { true }
            _ => { false }
        }
    }

    pub fn isArithmeticOp(&self) -> bool {
        match self {
            TCode::ADD => { true }
            TCode::SUB => { true }
            TCode::MULT => { true }
            TCode::DIV => { true }
            _ => { false }
        }
    }

    pub fn isLRAOp(&self) -> bool {
        self.isLogicalOp() || self.isRelationalOp() || self.isArithmeticOp()
    }
}


/// pair of numeric values of same type
#[derive(Debug, Clone)]
enum DNumPair {
    I64(i64,i64),
    F64(f64,f64),
}


/// dynamic value of Typ::D
///
/// Typ::D =
///   | ATyp::TOK | ATyp::BOOL | ATyp::CHAR | ATyp::I64 | ATyp::F64
///   | CTyp::FUNC (Typ::D -> Typ::D)
///   | CTyp::LIST (Tup::D)
///   | CTyp::MAP (Typ::D -> Typ::D)
///
#[derive(Debug, Clone)]
pub enum DValue {
    TOK,
    BOOL(bool),
    CHAR(char),
    I64(i64),
    F64(f64),
    FUNC(Rc<MTree>),
    _LIST(Vec<DValue>),
    _MAP(HashMap<DValue, DValue>),
}


impl DValue {

    pub fn dynamic_typ(&self) -> Typ {
        match self {
            DValue::TOK => { Typ::A(ATyp::TOK) }
            DValue::BOOL(_) => { Typ::A(ATyp::BOOL) }
            DValue::CHAR(_) => { Typ::A(ATyp::CHAR) }
            DValue::I64(_) => { Typ::A(ATyp::I64) }
            DValue::F64(_) => { Typ::A(ATyp::F64) }
            DValue::FUNC(_) => {
                Typ::C(CTyp::FUNC(Box::new(Typ::D), Box::new(Typ::D)))
            }
            DValue::_LIST(_) => {
                Typ::C(CTyp::LIST(Box::new(Typ::D)))
            }
            DValue::_MAP(_) => {
                Typ::C(CTyp::MAP(Box::new(Typ::D), Box::new(Typ::D)))
            }
        }
    }

    pub fn toString(&self) -> String {
        match self {
            DValue::TOK => { String::from("●") }
            DValue::BOOL(b) => { b.to_string() }
            DValue::CHAR(c) => { c.to_string() }
            DValue::I64(i) => { i.to_string() }
            DValue::F64(f) => { f.to_string() }
            DValue::FUNC(_) => { todo!() }  // what to do here?
            DValue::_LIST(_) => { todo!() }
            DValue::_MAP(_) => { todo!() }
        }
    }

    pub fn cast(&self, typ: ATyp) -> DValue {
        match self {
            DValue::TOK => {
                match typ {
                    ATyp::TOK => { self.clone() }
                    ATyp::BOOL => { panic!("Can't cast value {:?} to type {:?}", self, typ); }
                    ATyp::CHAR => { DValue::CHAR('●') }
                    ATyp::I64 => { panic!("Can't cast value {:?} to type {:?}", self, typ); }
                    ATyp::F64 => { panic!("Can't cast value {:?} to type {:?}", self, typ); }
                }
            }
            DValue::BOOL(b) => {
                match typ {
                    ATyp::TOK => { panic!("Can't cast value {:?} to type {:?}", self, typ); }
                    ATyp::BOOL => { self.clone() }
                    ATyp::CHAR => { DValue::CHAR(if *b { '⊤' } else { '⊥' }) }
                    ATyp::I64 => { DValue::I64(if *b { 1_i64 } else { 0_i64 }) }
                    ATyp::F64 => { DValue::F64(if *b { 1_f64 } else { 0_f64 }) }
                }
            }
            DValue::CHAR(c) => {
                match typ {
                    ATyp::TOK => { panic!("Can't cast value {:?} to type {:?}", self, typ); }
                    ATyp::BOOL => { panic!("Can't cast value {:?} to type {:?}", self, typ); }
                    ATyp::CHAR => { self.clone() }
                    ATyp::I64 => { DValue::I64(*c as i64) }
                    ATyp::F64 => { DValue::F64(*c as i64 as f64) }
                }
            }
            DValue::I64(i) => {
                match typ {
                    ATyp::TOK => { panic!("Can't cast value {:?} to type {:?}", self, typ); }
                    ATyp::BOOL => { panic!("Can't cast value {:?} to type {:?}", self, typ); }
                    ATyp::CHAR => { panic!("Can't cast value {:?} to type {:?}", self, typ); }
                    ATyp::I64 => { self.clone() }
                    ATyp::F64 => { DValue::F64(*i as f64) }
                }
            }
            DValue::F64(f) => {
                match typ {
                    ATyp::TOK => { panic!("Can't cast value {:?} to type {:?}", self, typ); }
                    ATyp::BOOL => { panic!("Can't cast value {:?} to type {:?}", self, typ); }
                    ATyp::CHAR => { panic!("Can't cast value {:?} to type {:?}", self, typ); }
                    ATyp::I64 => { DValue::I64(*f as i64) }
                    ATyp::F64 => { self.clone() }
                }
            }
            _ => { panic!("Not implemented!") }
        }
    }

    pub fn unaryOp(&self, code: TCode) -> DValue {
        match self {
            DValue::TOK => { panic!("Operator {:?} is undefined on DValue::TOK!", code) }
            DValue::BOOL(b) => {
                match code {
                    TCode::NOT => { DValue::BOOL(! *b) }
                    _ => { panic!("Operator {:?} is undefined on DValue::Bool!", code) }
                }
            }
            DValue::CHAR(_) => { panic!("Operator {:?} is undefined on DValue::CHAR!", code) }
            DValue::I64(i) => {
                match code {
                    TCode::SUB => { DValue::I64(0 - *i) }
                    _ => { panic!("Operator {:?} is undefined on DValue::I64!", code) }
                }
            }
            DValue::F64(f) => {
                match code {
                    TCode::SUB => { DValue::F64(0.0 - *f) }
                    TCode::DIV => { DValue::F64(1.0 / *f) }
                    _ => { panic!("Operator {:?} is undefined on DValue::F64!", code) }
                }
            }
            DValue::FUNC(_) => { panic!("Operator {:?} is undefined on DValue::FUNC!", code) }
            DValue::_LIST(_) => { panic!("Operator {:?} is undefined on DValue::LIST!", code) }
            DValue::_MAP(_) => { panic!("Operator {:?} is undefined on DValue::MAP!", code) }
        }
    }


    pub fn binaryOp(&self, code: TCode, value_rhs : DValue) -> DValue {

        if code.isArithmeticOp() {
            self.arithmeticOp(code, value_rhs)
        } else if code.isRelationalOp() {
            self.relationalOp(code, value_rhs)
        } else if code.isLogicalOp() {
            self.logicalOp(code, value_rhs)
        } else {
            panic!("Binary operator {:?} is undefined on values ({:?},{:?})!", code, self, value_rhs)
        }

    }


    fn commonDNumPair(&self, code: &TCode, value_rhs : DValue) -> DNumPair {
        let dt_lhs = self.dynamic_typ();
        let dt_rhs = value_rhs.dynamic_typ();

        if !(dt_lhs.isNumeric() && dt_rhs.isNumeric()) {
            panic!("Both types need to be numeric for operator {:?}!", code)
        }
        let (Typ::A(at_lhs), Typ::A(at_rhs)) = (dt_lhs, dt_rhs) else {
            panic!("Both types need to be atomic!")
        };

        let adt_common = at_lhs.getCommon(at_rhs);
        let dv_lhs = self.cast(adt_common);
        let dv_rhs = value_rhs.cast(adt_common);

        match adt_common {
            ATyp::I64 => {
                if let (DValue::I64(lhs),DValue::I64(rhs)) = (dv_lhs, dv_rhs) {
                    DNumPair::I64(lhs, rhs)
                } else { panic!("") }
            }
            ATyp::F64 => {
                if let (DValue::F64(lhs),DValue::F64(rhs)) = (dv_lhs, dv_rhs) {
                    DNumPair::F64(lhs, rhs)
                } else { panic!("") }
            }
            _ => { panic!("") }
        }
    }


    fn arithmeticOp(&self, code: TCode, value_rhs : DValue) -> DValue {

        let num_pair = self.commonDNumPair(&code, value_rhs);

        match code {
            TCode::ADD => {
                match num_pair {
                    DNumPair::I64(l, r) => { DValue::I64(l + r) }
                    DNumPair::F64(l, r) => { DValue::F64(l + r) }
                }
            }
            TCode::SUB =>  {
                match num_pair {
                    DNumPair::I64(l, r) => { DValue::I64(l - r) }
                    DNumPair::F64(l, r) => { DValue::F64(l - r) }
                }
            }
            TCode::MULT => {
                match num_pair {
                    DNumPair::I64(l, r) => { DValue::I64(l * r) }
                    DNumPair::F64(l, r) => { DValue::F64(l * r) }
                }
            }
            TCode::DIV => {
                match num_pair {
                    DNumPair::I64(l, r) => { DValue::I64(l / r) }
                    DNumPair::F64(l, r) => { DValue::F64(l / r) }
                }
            }
            _ => { panic!("{:?} is not arithmetic operator!", code) }
        }
    }


    fn relationalOp(&self, code: TCode, value_rhs : DValue) -> DValue {

        let num_pair = self.commonDNumPair(&code, value_rhs);

        match code {
            TCode::LT => {
                match num_pair {
                    DNumPair::I64(l, r) => { DValue::BOOL(l < r) }
                    DNumPair::F64(l, r) => { DValue::BOOL(l < r) }
                }
            }
            TCode::GT => {
                match num_pair {
                    DNumPair::I64(l, r) => { DValue::BOOL(l > r) }
                    DNumPair::F64(l, r) => { DValue::BOOL(l > r) }
                }
            }
            TCode::EQ => {
                match num_pair {
                    DNumPair::I64(l, r) => { DValue::BOOL(l == r) }
                    DNumPair::F64(l, r) => { DValue::BOOL(l == r) }
                }
            }
            TCode::NOT_EQ => {
                match num_pair {
                    DNumPair::I64(l, r) => { DValue::BOOL(l != r) }
                    DNumPair::F64(l, r) => { DValue::BOOL(l != r) }
                }
            }
            _ => {  panic!("{:?} is not relational operator!", code) }
        }
    }


    fn logicalOp(&self, code: TCode, value_rhs : DValue) -> DValue {

        let b_lhs = match self {
            DValue::BOOL(b) => { *b }
            _ => { panic!("Logical operator requires LHS value to be of type Bool!") }
        };
        let b_rhs = match value_rhs {
            DValue::BOOL(b) => { b }
            _ => { panic!("Logical operator requires RHS value to be of type Bool!") }
        };

        match code {
            TCode::OR => { DValue::BOOL(b_lhs || b_rhs) }
            TCode::AND => { DValue::BOOL(b_lhs && b_rhs) }
            _ => { panic!("{:?} is not logical operator!", code) }
        }
    }

}