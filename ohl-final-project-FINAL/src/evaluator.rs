use std::cell::RefCell;
use std::ops::{Deref};
use std::rc::Rc;
use crate::frame_analyze::{FrameTyp};
use crate::frame_call::CFrame;
use crate::log::Log;
use crate::mtree::MTree;
use crate::token::{TCode};
use crate::value::{DValue};


// control flow
pub enum Control {
    NEXT,
    RETURN,
    _BREAK,
    _CONTINUE,
}


/// evaluate an analyzed MTree
pub struct Evaluator {
    pub log: Log,
}


impl Evaluator {

    pub fn new() -> Evaluator {
        Evaluator {
            log: Log::new(),
        }
    }


    pub fn evaluate(&mut self, mtree_block: &MTree) {
        self.log.show_debug = false;
        self.evaluate_block(mtree_block, None);
    }


    pub fn evaluate_block(
        &mut self, mtree_block: &MTree, option_frame_up: Option<Rc<RefCell<CFrame>>>)
        -> (DValue, Control)
    {
        // get block's AFrame
        let rcc_aFrame = match &mtree_block.token.code {
            TCode::A_BLOCK(rcc_aFrame) => { rcc_aFrame }
            _ => {  panic!("Expected Code::META_BLOCK tree!") }
        };

        // create new dynamic block frame (CFrame)
        let size_cFrame = rcc_aFrame.borrow().size_symbols(FrameTyp::FUNCTION);
        let mut cFrame = CFrame::new(size_cFrame);
        cFrame.aFrame = Some(rcc_aFrame.clone()); // link with AFrame
        cFrame.cFrame_up = option_frame_up; // link with OUTER CFrame

        // evaluate block with already created INNER BLOCK CFrame
        let rcc_frame_block = Rc::new(RefCell::new(cFrame));
        self.evaluate_block_framed(mtree_block, rcc_frame_block)
    }


    pub fn evaluate_block_framed(
        &mut self, mtree_block: &MTree, rcc_frame_block: Rc<RefCell<CFrame>>)
        -> (DValue, Control)
    {
        self.log.debug("evaluate2_block()");
        self.log.indent_inc();
        let mut ret = (DValue::TOK, Control::NEXT);

        // evaluate all statements in block
        for child in mtree_block.children.iter() {
            let mtree_stmt = child.deref();
            ret = self.evaluate_stmt(mtree_stmt, rcc_frame_block.clone());
            match ret.1 {
                Control::NEXT => { continue; }
                Control::_BREAK => { ret.1 = Control::NEXT; break; }
                _ => { break; }
            }
        }

        self.log.indent_dec();
        ret
    }


    pub fn evaluate_stmt(&mut self, mtree_stmt: &MTree, rcc_frame: Rc<RefCell<CFrame>>)
                         -> (DValue, Control)
    {
        match &mtree_stmt.token.code {
            TCode::KW_RETURN => {
                self.evaluate_return(mtree_stmt, rcc_frame)
            }
            TCode::KW_IF => {
                self.evaluate_if(mtree_stmt, rcc_frame)
            }
            TCode::KW_WHILE => {
                self.evaluate_while(mtree_stmt, rcc_frame)
            }
            TCode::KW_READ => {
                (self.evaluate_read(mtree_stmt, rcc_frame), Control::NEXT)
            }
            TCode::KW_WRITE => {
                (self.evaluate_write(mtree_stmt, rcc_frame), Control::NEXT)
            }
            TCode::FUNC => {
                (DValue::TOK, Control::NEXT)
            }
            _ => {
                // assume tree is an expression
                (self.evaluate_expr(mtree_stmt, rcc_frame), Control::NEXT)
            }
        }
    }


    pub fn evaluate_return(
        &mut self, mtree_return: &MTree, rcc_frame: Rc<RefCell<CFrame>>)
        -> (DValue, Control)
    {
        self.log.debug("evaluate_return()");
        self.log.indent_inc();
        let mtree_expr = mtree_return.children.get(0).unwrap().deref();
        let value = self.evaluate_expr(mtree_expr, rcc_frame);
        self.log.debug(format!("value={:?}", value).as_str());
        self.log.indent_dec();
        (value, Control::RETURN)
    }


    pub fn evaluate_if(
        &mut self, mtree_if: &MTree, rcc_frame: Rc<RefCell<CFrame>>)
        -> (DValue, Control)
    {
        self.log.debug("evaluate_if()");
        self.log.indent_inc();
        let cond = mtree_if.children.get(0).unwrap().deref();
        let value_cond = self.evaluate_expr(cond, rcc_frame.clone());
        let ret_block = if let DValue::BOOL(b) = value_cond {
            let idx_branch = if b { 1 } else { 2 };
            let mtree_branch = mtree_if.children.get(idx_branch).unwrap().deref();
            self.evaluate_block(mtree_branch, Some(rcc_frame.clone()))
        } else {
            panic!("Condition must result in value of type Bool!");
        };
        self.log.indent_dec();
        ret_block
    }

    pub fn evaluate_while(
        &mut self, _mtree_while: &MTree, _rcc_frame: Rc<RefCell<CFrame>>)
        -> (DValue, Control)
    {
        todo!()
    }


    pub fn evaluate_read(
        &mut self, _mtree_read: &MTree, _rcc_frame: Rc<RefCell<CFrame>>)
        -> DValue
    {
        todo!()
    }


    pub fn evaluate_write(
        &mut self, mtree_write: &MTree, rcc_frame: Rc<RefCell<CFrame>>)
        -> DValue
    {
        self.log.debug("evaluate_print()");
        self.log.indent_inc();
        let mtree_expr = mtree_write.children.get(0).unwrap().deref();
        let value = self.evaluate_expr(mtree_expr, rcc_frame);
        print!("> {:}\n", value.toString());
        self.log.indent_dec();
        value
    }


    pub fn evaluate_expr(
        &mut self, mtree_expr: &MTree, rcc_frame: Rc<RefCell<CFrame>>)
        -> DValue
    {
        self.log.debug("evaluate_expr()");
        self.log.indent_inc();
        let code = mtree_expr.token.code.clone();

        let value = if let TCode::A_REF(loc) = &code {

            self.log.debug(format!("LOC {:?}", loc).as_str() );
            let value = rcc_frame.borrow().value_load(loc);
            self.log.debug(format!("VAL {:?}", value).as_str() );
            value

        } else if let TCode::VAL(val) = code {

            val.clone()

        } else if let TCode::CALL = code {

            self.evaluate_call(mtree_expr, rcc_frame.clone())

        } else if code.isLRAOp() {

            if mtree_expr.children.len() == 1 {

                let mtree_unary= mtree_expr.children.get(0).unwrap().deref();
                let value_unary = self.evaluate_expr(
                    mtree_unary, rcc_frame.clone());
                value_unary.unaryOp(code)

            } else if mtree_expr.children.len() == 2 {

                let mtree_left= mtree_expr.children.get(0).unwrap().deref();
                let mtree_right= mtree_expr.children.get(1).unwrap().deref();
                let value_left = self.evaluate_expr(
                    mtree_left, rcc_frame.clone());
                let value_right = self.evaluate_expr(
                    mtree_right, rcc_frame.clone());
                value_left.binaryOp(code, value_right)

            } else {
                panic!();
            }

        } else if let TCode::OP_ASSIGN = &code {

            // get storage location (LHS)
            let mtree_left= mtree_expr.children.get(0).unwrap().deref();
            let loc_left = match  & mtree_left.token.code {
                TCode::A_REF(loc) => { loc.clone() }
                _ => { panic!("Left operand of assignment must be REF!"); }
            };

            // get value (RHS)
            let mtree_right= mtree_expr.children.get(1).unwrap().deref();
            let value_right = self.evaluate_expr(
                mtree_right, rcc_frame.clone());

            // assign value to storage location
            rcc_frame.borrow_mut().value_store(&loc_left, value_right.clone());
            value_right

        } else {
            panic!("Code: {:?}", code)
        };

        self.log.debug(format!("value={:?}", value).as_str());
        self.log.indent_dec();
        value
    }


    pub fn evaluate_call(
        &mut self, mtree_call: &MTree, rcc_frame: Rc<RefCell<CFrame>>)
        -> DValue
    {
        self.log.debug("evaluate_call()");
        self.log.indent_inc();

        // find function code
        let mtree_ref = mtree_call.children.get(0).unwrap().deref();
        let mtree_func = match &mtree_ref.token.code {
            TCode::A_REF(loc) => {
                let value = rcc_frame.borrow().value_load(loc);
                if let DValue::FUNC(rc_func) = value {
                    rc_func
                } else {
                    panic!("Expected DValue::FUNC!")
                }
            }
            _ => { panic!("Expected REF!") }
        };

        // create new call frame (CFrame) and link with current call frame
        let n_args = mtree_call.children.len() - 1; // number of call arguments
        let cFrame = CFrame::new(n_args);
        let rcc_frame_func = Rc::new(RefCell::new(cFrame));
        rcc_frame_func.borrow_mut().cFrame_up = rcc_frame.borrow().cFrame_up.clone();

        // evaluate positional arguments 0,1,2,.. and store in frame
        for (idx, child) in mtree_call.children.iter().enumerate() {
            if idx == 0 {
                continue;
            }
            let idx_arg = idx - 1;
            let mtree_arg = child.deref();
            let value_arg = self.evaluate_expr(
                mtree_arg, rcc_frame.clone()
            );
            rcc_frame_func.borrow_mut().value_store_cell(idx_arg, value_arg)
        }

        // evaluate function
        let value = self.evaluate_func(mtree_func.deref(), rcc_frame_func);

        self.log.indent_dec();
        value
    }


    pub fn evaluate_func(
        &mut self, mtree_func: &MTree, rcc_frame_func: Rc<RefCell<CFrame>>)
        -> DValue
    {
        self.log.debug("evaluate_func()");
        self.log.indent_inc();

        // link static block frame for (dynamic) lookup
        let mtree2_block = mtree_func.children.get(0).unwrap().deref();
        if let TCode::A_BLOCK(rcc_statics) = &mtree2_block.token.code {
            rcc_frame_func.borrow_mut().aFrame = Some(rcc_statics.clone());
        } else {
            panic!("Expected Code::META_BLOCK tree!")
        }

        // evaluate block with already created FUNCTION frame
        let (value, _) = self.evaluate_block_framed(mtree2_block, rcc_frame_func);

        self.log.indent_dec();
        value
    }

}