use std::cell::RefCell;
use std::rc::Rc;
use crate::frame_analyze::{AFrame, CellLoc, FrameTyp};
use crate::value::DValue;


/// A (stack) frame for function calls (invocations)
#[derive(Debug, Clone)]
pub struct CFrame {
    pub aFrame: Option<Rc<RefCell<AFrame>>>,     // link to static frame
    pub cFrame_up: Option<Rc<RefCell<CFrame>>>,  // link to last stack frame
    dValues: Vec<DValue>,                        // values in stack frame
}


impl CFrame {

    pub fn new(size: usize) -> CFrame {
        CFrame {
            aFrame: None,
            cFrame_up: None,
            dValues: vec![DValue::TOK; size],
        }
    }


    /// load a value relative to this frame
    pub fn value_load(&self, loc: &CellLoc) -> DValue {
        match loc.typ {
            FrameTyp::PROGRAM => { // values with 'process' lifetime
                match &self.aFrame {
                    Some(rcc_aFrame) => {
                        rcc_aFrame.borrow().value_load(loc)
                    }
                    None => { panic!("AFrame is missing!") }
                }
            }
            FrameTyp::FUNCTION => { // values on call stack
                if loc.idx_frame == 0 {
                    self.dValues.get(loc.idx_cell).unwrap().clone()
                } else {
                    match &self.cFrame_up {
                        Some(rcc_up) => {
                            let loc_up = loc.idx_store_dec();
                            rcc_up.borrow().value_load(&loc_up)
                        }
                        None => { panic!("CFrame up is missing!") }
                    }
                }
            }
        }
    }


    /// store a value in this call frame
    pub fn value_store_cell(&mut self, idx_cell: usize, value: DValue) {
        self.dValues.insert(idx_cell, value);
    }


    /// store a value relative to this frame
    pub fn value_store(&mut self, loc: &CellLoc, value: DValue) {
        match loc.typ {
            FrameTyp::PROGRAM => { // values with 'process' lifetime
                match &self.aFrame {
                    Some(rcc_aFrame) => {
                        rcc_aFrame.borrow_mut().value_store(loc, value);
                    }
                    None => { panic!("AFrame is missing!") }
                }
            }
            FrameTyp::FUNCTION => { // values on call stack
                if loc.idx_frame == 0 {
                    self.dValues.insert(loc.idx_cell, value);
                } else {
                    match &self.cFrame_up {
                        Some(rcc_up) => {
                            let loc_up = loc.idx_store_dec();
                            rcc_up.borrow_mut().value_store(&loc_up, value);
                        }
                        None => { panic!("CFrame up is missing!") }
                    }
                }
            }
        }
    }

}