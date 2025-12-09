#![allow(dead_code)]

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use crate::typ::{Typ};
use crate::value::DValue;


/// type of memory binding:
///
///   PROGRAM: 'process' / static lifetime
///   FUNCTION: 'function call' / call stack frame lifetime
///
#[derive(Debug, Copy, Clone)]
pub enum FrameTyp {
    PROGRAM = 0,
    FUNCTION = 1,
}


/// location of 'memory cell' for values
#[derive(Debug, Copy, Clone)]
pub struct CellLoc {
    pub typ: FrameTyp,      // type of frame 'process' / 'function call'
    pub idx_frame: usize,   // relative idx of frame (0,1,2, ... up)
    pub idx_cell: usize,    // index of cell in frame
}


impl CellLoc {

    pub fn new(typ: FrameTyp) -> CellLoc {
        CellLoc { typ, idx_frame: 0, idx_cell: 0 }
    }

    pub fn idx_store_dec(&self) -> CellLoc {
        CellLoc {
            typ: self.typ,
            idx_frame: self.idx_frame - 1,
            idx_cell: self.idx_cell,
        }
    }
}


/// symbol information for code analysis
/// (is currently also used during evaluation for (static) variables with FrameTyp::PROGRAM)
#[derive(Debug, Clone)]
pub struct ASymbol {
    pub name: String,           // name of the symbol
    pub mutable: bool,          // can symbol be assigned ?
    pub typ: Typ,               // symbol type (currently Typ::D for dynamic value)
    pub loc: CellLoc,           // symbol location in some frame
    pub value: DValue,          // current symbol value
}


impl ASymbol {

    pub fn new(name: String, storeTyp: FrameTyp) -> ASymbol {
        ASymbol {
            name,
            loc: CellLoc::new(storeTyp),
            mutable: false,
            typ: Typ::D,
            value: DValue::TOK,
        }
    }
}


/// A frame for code analysis
#[derive(Debug, Clone)]
pub struct AFrame {
    pub frame_up: Option<Rc<RefCell<AFrame>>>,      // parent frame
    name2idx: HashMap<String, (FrameTyp, usize)>,   // maps name to symbol entry
    symbols_program: Vec<ASymbol>,                  // symbols per program instance (process)
    symbols_function: Vec<ASymbol>,                 // symbols per function instance
}


impl AFrame {

    const IDX_PROC : usize = 0;
    const IDX_CALL : usize = 1;


    pub fn new() -> AFrame {
        AFrame {
            frame_up: Option::None,
            name2idx: HashMap::new(),
            symbols_program: vec![],
            symbols_function: vec![],
        }
    }


    pub fn new_child(frame_parent: Rc<RefCell<AFrame>>) -> AFrame {
        let mut frame_child = Self::new();
        frame_child.frame_up = Some(frame_parent);
        frame_child
    }


    pub fn symbol_new(&mut self, mut symbol: ASymbol) -> CellLoc {
        let idx_cell = match symbol.loc.typ {
            FrameTyp::PROGRAM => { self.symbols_program.len() }
            FrameTyp::FUNCTION => { self.symbols_function.len() }
        };
        let loc = CellLoc {
            typ: symbol.loc.typ,
            idx_frame: 0,
            idx_cell,
        };
        symbol.loc = loc;
        self.name2idx.insert(symbol.name.clone(), (loc.typ, loc.idx_cell));
        match symbol.loc.typ {
            FrameTyp::PROGRAM => { self.symbols_program.push(symbol); }
            FrameTyp::FUNCTION => { self.symbols_function.push(symbol); }
        }
        loc
    }

    pub fn symbol_access_here(&mut self, loc: CellLoc) -> &mut ASymbol {
        match loc.typ {
            FrameTyp::PROGRAM => { self.symbols_program.get_mut(loc.idx_cell).unwrap() }
            FrameTyp::FUNCTION => { self.symbols_function.get_mut(loc.idx_cell).unwrap() }
        }
    }

    pub fn symbol_lookup(&self, name: &String) -> Option<ASymbol> {
        self.symbol_lookup_up(name, 0)
    }


    fn symbol_lookup_up(&self, name: &String, idx_store: usize) -> Option<ASymbol> {

        if let Some( (storeTyp, idx_cell) ) = self.name2idx.get(name) {
            let symbol = match storeTyp {
                FrameTyp::PROGRAM => { self.symbols_program.get(*idx_cell).unwrap() }
                FrameTyp::FUNCTION => { self.symbols_function.get(*idx_cell).unwrap() }
            };
            let mut symbol_ = symbol.clone();
            symbol_.loc.idx_frame = idx_store;
            Some(symbol_)
        } else {
            if let Some(rc_up) = &self.frame_up {
                rc_up.borrow().symbol_lookup_up(name, idx_store + 1)
            } else {
                None
            }
        }
    }


    pub fn size_symbols(&self, typ: FrameTyp) -> usize {
        match typ {
            FrameTyp::PROGRAM => { self.symbols_program.len() }
            FrameTyp::FUNCTION => { self.symbols_function.len() }
        }
    }


    pub fn value_load(&self, loc: &CellLoc) -> DValue {
        if loc.idx_frame == 0 {
            let symbols = match loc.typ {
                FrameTyp::PROGRAM => { &self.symbols_program }
                FrameTyp::FUNCTION => { &self.symbols_function }
            };
            match symbols.get(loc.idx_cell) {
                Some(aSymbol) => {
                    aSymbol.value.clone()
                }
                None => { panic!("Can't find symbol at {:?}", loc) }
            }

        } else {
            match &self.frame_up {
                Some(rcc_frame) => {
                    let loc = loc.idx_store_dec();
                    rcc_frame.borrow_mut().value_load(&loc)
                }
                None => { panic!("AFrame up is missing!")}
            }
        }
    }


    pub fn value_store_init(&mut self, loc: &CellLoc, value: DValue, init: bool) {
        if loc.idx_frame == 0 {
            let symbols = match loc.typ {
                FrameTyp::PROGRAM => { &mut self.symbols_program }
                FrameTyp::FUNCTION => { &mut self.symbols_function }
            };
            match symbols.get_mut(loc.idx_cell) {
                Some(aSymbol) => {
                    if aSymbol.mutable || init {
                        aSymbol.value = value;
                    } else {
                        panic!("Can't store to immutable cell!")
                    }
                }
                None => { panic!("Can't find symbol at {:?}", loc) }
            }

        } else {
            match &self.frame_up {
                Some(rcc_frame) => {
                    let loc = loc.idx_store_dec();
                    rcc_frame.borrow_mut().value_store_init(&loc, value, init);
                }
                None => { panic!("AFrame up is missing!")}
            }
        }
    }


    pub fn value_store(&mut self, loc: &CellLoc, value: DValue) {
        self.value_store_init(loc, value, false);
    }


    pub fn print(&self) -> String {
        let mut s = String::new();
        for (name, (storeTyp, idx_cell)) in &self.name2idx {
            s = format!("{:}\n| SYMBOL {:?} #{:} {:?}", s, storeTyp, idx_cell, name);
        }
        s
    }
}


