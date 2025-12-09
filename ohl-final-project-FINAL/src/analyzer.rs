use std::cell::RefCell;
use std::ops::{Deref};
use std::rc::Rc;
use crate::frame_analyze::{AFrame, ASymbol, FrameTyp};
use crate::log::Log;
use crate::mtree::MTree;
use crate::token::{TCode, Token};
use crate::value::DValue;


pub struct Analyzer {
    pub log: Log,
}

impl Analyzer {

    pub fn new() -> Analyzer {
        Analyzer {
            log: Log::new(),
        }
    }

    pub fn analyze_global(&self, rc_mtree_global : Rc<MTree>) -> Rc<MTree> {
        let rcc_frame = Rc::new(RefCell::new(AFrame::new()));
        self.analyze_block(rc_mtree_global.deref(), rcc_frame)
    }

    pub fn analyze_block(
        &self, mtree_block: &MTree, rcc_frame_parent: Rc<RefCell<AFrame>>)
        -> Rc<MTree>
    {
        // create AFrame
        let frame_block = AFrame::new_child(rcc_frame_parent);
        let rcc_frame_block = Rc::new(RefCell::new(frame_block));

        self.analyze_block_framed(mtree_block, rcc_frame_block)
    }

    pub fn analyze_block_framed(&self, mtree_block: &MTree, rcc_frame_block: Rc<RefCell<AFrame>>) -> Rc<MTree> {

        // collect symbols
        let token_block_ = Token::from(TCode::A_BLOCK(rcc_frame_block.clone()));
        let mut tree_block_ = MTree::new(token_block_);

        for rc_stmt in &mtree_block.children {
            let rc_stmt_ = self.analyze_stmt(
                rc_stmt.deref(),
                rcc_frame_block.clone()
            );
            tree_block_.children.push(rc_stmt_)
        };

        Rc::new(tree_block_)
    }

    pub fn analyze_stmt(&self, tree_stmt: &MTree, rcc_frame: Rc<RefCell<AFrame>>) -> Rc<MTree> {

        match tree_stmt.token.code {
            TCode::FUNC => { self.analyze_func(tree_stmt, rcc_frame) }
            TCode::KW_IF => { self.analyze_if(tree_stmt, rcc_frame) }
            TCode::KW_WHILE => { self.analyze_while(tree_stmt, rcc_frame) }
            TCode::KW_RETURN => { self.analyze_return(tree_stmt, rcc_frame) }
            TCode::OP_ASSIGN => { self.analyze_assign(tree_stmt, rcc_frame) }
            TCode::A_BLOCK(_) => { self.analyze_block(tree_stmt, rcc_frame) }
            TCode::KW_READ => { self.analyze_read(tree_stmt, rcc_frame) }
            TCode::KW_WRITE => { self.analyze_write(tree_stmt, rcc_frame) }
            TCode::CALL => { self.analyze_call(tree_stmt, rcc_frame) }
            _ => {
                panic!("Code {:?}", tree_stmt.token.code);
            }
        }

    }

    pub fn analyze_func(&self, mtree_func: &MTree, rcc_frame: Rc<RefCell<AFrame>>) -> Rc<MTree> {


        // create new block frame (statics)
        let mut frame_func_ = AFrame::new_child(rcc_frame.clone());

        // record name of function
        let mtree_id = mtree_func.children.get(0).unwrap().deref();
        let idx_func = if let TCode::ID(name_func) = & mtree_id.token.code {

            self.log.debug(format!("analyze_func() '{:}'", name_func).as_str());

            let rc = Rc::new(MTree::new( mtree_id.token.clone()));
            let mut symbol = ASymbol::new(name_func.clone(), FrameTyp::PROGRAM);
            symbol.value = DValue::FUNC(rc); // placeholder
            rcc_frame.borrow_mut().symbol_new(symbol)
        } else {
            panic!("Missing function ID in FUNC MTree!")
        };

        // collect symbols
        let mtree_params = mtree_func.children.get(1).unwrap().deref();
        for rc_mtree_param in & mtree_params.children {
            let mtree_param = rc_mtree_param.deref();
            match & mtree_param.token.code {
                TCode::ID(name) => {
                    let symbol = ASymbol::new(name.clone(), FrameTyp::FUNCTION);
                    frame_func_.symbol_new(symbol);
                }
                _ => { panic!("Illegal Params Tree") }
            }
        }

        // analyze block
        let rc_block = mtree_func.children.get(2).unwrap().deref();
        let rc_block_ = self.analyze_block_framed(
            rc_block,
            Rc::new(RefCell::new(frame_func_))
        );

        // create new tree node
        let mut mtree_func_ = MTree::new(Token::from(TCode::FUNC));
        mtree_func_.children.insert(0, rc_block_);
        let rc_mtree_func_ = Rc::new(mtree_func_);

        // set code (value) of function
        rcc_frame.borrow_mut().symbol_access_here( idx_func ).value =
            DValue::FUNC(rc_mtree_func_.clone());

        // return func
        return rc_mtree_func_;
    }

    pub fn analyze_if(&self, mtree_if: &MTree, frame: Rc<RefCell<AFrame>>) -> Rc<MTree> {

        let mtree_cond = mtree_if.children.get(0).unwrap().deref();
        let mtree_then = mtree_if.children.get(1).unwrap().deref();
        let mtree_else = mtree_if.children.get(2).unwrap().deref();
        let rc_cond = self.analyze_expr(mtree_cond, frame.clone());
        let rc_true = self.analyze_block(mtree_then, frame.clone());
        let rc_false = self.analyze_block(mtree_else, frame.clone());

        let token_if = Token {
            code: TCode::KW_IF,
            loc: mtree_if.token.loc.clone(),
        };
        let mut mtree_if_ = MTree::new( token_if);
        mtree_if_.children.push(rc_cond);
        mtree_if_.children.push(rc_true);
        mtree_if_.children.push(rc_false);

        Rc::new(mtree_if_)
    }

    pub fn analyze_while(&self, _mtree_while: &MTree, _frame: Rc<RefCell<AFrame>>) -> Rc<MTree> {
        todo!()
    }

    pub fn analyze_return(&self, mtree_return: &MTree, frame: Rc<RefCell<AFrame>>) -> Rc<MTree> {
        let rc_expr = mtree_return.children.get(0).unwrap();
        self.analyze_expr(rc_expr.deref(), frame)
    }

    pub fn analyze_assign(&self, mtree_assign: &MTree, rcc_frame: Rc<RefCell<AFrame>>) -> Rc<MTree> {

        // expr on RHS
        let mtree_expr = mtree_assign.children.get(1).unwrap().deref();
        let rc_mtree_expr_ = self.analyze_expr(mtree_expr, rcc_frame.clone());

        // symbol on LHS
        let mtree_id = mtree_assign.children.get(0).unwrap().deref();
        let loc = if let TCode::ID(name) = & mtree_id.token.code {
            // lookup or create symbol
            let option_symbol = rcc_frame.borrow().symbol_lookup(name);
            if let Some(symbol) = option_symbol {
                symbol.loc
            } else {
                let symbol = ASymbol::new(name.clone(), FrameTyp::FUNCTION);
                rcc_frame.borrow_mut().symbol_new(symbol)
            }
        } else {
            panic!("Expect identifier on LHS of assignment!");
        };
        let token_ref = Token {
            code: TCode::A_REF(loc.clone()),
            loc: mtree_id.token.loc.clone(),
        };
        let rc_mtree_ref = Rc::new(MTree::new( token_ref ));

        // new MTree
        let rc_mtree_assign_ = Rc::new(
            MTree {
                token: mtree_assign.token.clone(),
                children: vec![ rc_mtree_ref, rc_mtree_expr_ ],
            }
        );
        rc_mtree_assign_
    }

    pub fn analyze_write(&self, mtree_write: &MTree, frame: Rc<RefCell<AFrame>>) -> Rc<MTree> {
        let rc_mtree_expr = mtree_write.children.get(0).unwrap();
        let rc_mtree_expr_ = self.analyze_expr(rc_mtree_expr.deref(), frame);
        Rc::new(MTree {
            token: mtree_write.token.clone(),
            children: vec![rc_mtree_expr_],
        })
    }

    pub fn analyze_read(&self, mtree_read: &MTree, frame: Rc<RefCell<AFrame>>) -> Rc<MTree> {
        let rc_mtree_id = mtree_read.children.get(0).unwrap();
        let rc_mtree_id_ = self.analyze_id_load(rc_mtree_id.deref(), frame);
        Rc::new(MTree {
            token: mtree_read.token.clone(),
            children: vec![rc_mtree_id_],
        })
    }

    pub fn analyze_expr(&self, mtree_expr: &MTree, frame: Rc<RefCell<AFrame>>) -> Rc<MTree> {
        let code = &mtree_expr.token.code;
        match code {
            TCode::ID(_name) => {
                self.analyze_id_load(mtree_expr, frame)
            }
            TCode::VAL(_val) => {
                Rc::new(mtree_expr.clone())
            }
            TCode::CALL => {
                self.analyze_call(mtree_expr, frame)
            }
            _ => {
                if code.isLRAOp() {
                    self.analyze_LRAOp(mtree_expr, frame)
                } else {
                    panic!("Code {:?}", code)
                }
            }
        }
    }

    pub fn analyze_id_load(&self, mtree_id: &MTree, frame: Rc<RefCell<AFrame>>) -> Rc<MTree> {
        match &mtree_id.token.code {
            TCode::ID(name) => {
                // create reference into frame
                let symbol = match frame.borrow().symbol_lookup(name) {
                    Some(symbol) => { symbol }
                    None => { panic!("Can't find symbol for {:?}!", mtree_id.token); }
                };
                Rc::new(MTree::new( Token {
                    code: TCode::A_REF(symbol.loc.clone()),
                    loc: mtree_id.token.loc.clone(),
                }))
            }
            _ => {
                panic!("Expected ID but got {:?}", mtree_id.token.code)
            }
        }
    }

    pub fn analyze_call(&self, mtree_call: &MTree, frame: Rc<RefCell<AFrame>>) -> Rc<MTree> {

        let mut mtree_call_ = MTree::new(mtree_call.token.clone());
        for rc_arg_mtree in &mtree_call.children {
            let mtree_arg = rc_arg_mtree.deref();
            let rc_mtree_arg_ = self.analyze_expr(mtree_arg, frame.clone());
            mtree_call_.children.push(rc_mtree_arg_);
        }

        Rc::new(mtree_call_)
    }

    pub fn analyze_LRAOp(&self, mtree_expr: &MTree, frame: Rc<RefCell<AFrame>>) -> Rc<MTree> {
        let code = &mtree_expr.token.code;
        if mtree_expr.children.len() == 1 {
            let rc_arg = mtree_expr.children.get(0).unwrap();
            let rc_arg_ = self.analyze_expr(rc_arg.deref(), frame);
            let mut mtree_ = MTree::new(Token::from(code.clone()));
            mtree_.children.push(rc_arg_);
            Rc::new(mtree_)
        } else if mtree_expr.children.len() == 2 {
            let rc_arg0 = mtree_expr.children.get(0).unwrap();
            let rc_arg1 = mtree_expr.children.get(1).unwrap();
            let rc_arg0_ = self.analyze_expr(rc_arg0.deref(), frame.clone());
            let rc_arg1_ = self.analyze_expr(rc_arg1.deref(), frame.clone());
            let mut mtree_ = MTree::new(Token::from(code.clone()));
            mtree_.children.push(rc_arg0_);
            mtree_.children.push(rc_arg1_);
            Rc::new(mtree_)
        } else {
            panic!()
        }
    }


}