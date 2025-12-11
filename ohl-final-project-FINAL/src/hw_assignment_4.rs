#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(non_snake_case)]

use crate::hw_assignment_3::*;
use crate::token::*;
use crate::value::DValue;
use crate::mtree::*;

const INDENT: usize = 2;

pub struct Parser {
    pub(crate) lexer: Lexer,
    pub(crate) indent: usize,
}

impl Parser {
    pub fn new(mut lexer: Lexer) -> Parser {
        lexer.advance();
        Parser { lexer, indent: 0 }
    }

    pub fn analyze(&mut self) -> MTree {
        self.indent = 0;
        let tree = self.parse_program();
        self.expect(TCode::EOI);
        tree
    }
}
// Utility methods
impl Parser {
    pub fn curr(&self) -> TCode {
        self.lexer.curr()
    }

    pub fn advance(&mut self) {
        self.lexer.advance();
    }

    pub fn peek(&self, symbol: TCode) -> bool {
        self.curr() == symbol
    }

    pub fn expect(&mut self, symbol: TCode) {
        if self.curr() == symbol {
            self.advance();
            println!("{:<indent$}expect({symbol:?})", "", indent = self.indent);
        } else {
            panic!("Expected {:?}, got {:?}", symbol, self.curr());
        }
    }

    pub fn accept(&mut self, symbol: TCode) -> bool {
        if self.curr() == symbol {
            self.advance();
            true
        } else {
            false
        }
    }

    fn peek_id(&self, name: &str) -> bool {
        matches!(&self.curr(), TCode::ID(s) if s == name)
    }

    fn expect_id(&mut self, name: &str) {
        match self.curr() {
            TCode::ID(s) if s == name => self.advance(),
            _ => panic!("Expected identifier '{}'", name),
        }
    }
}

// Pretty printing
impl Parser {
    fn indent_print(&mut self, msg: &'static str) {
        println!("{:<width$}{}", "", msg, width = self.indent);
    }

    fn indent_inc(&mut self) {
        self.indent += INDENT;
    }

    fn indent_dec(&mut self) {
        self.indent -= INDENT;
    }
}

// Recursive descent parser
impl Parser {
    // program = { func }
    pub fn parse_program(&mut self) -> MTree {
        self.indent_print("parse_program()");
        self.indent_inc();
        let mut global = MTree::new(Token::from(TCode::BLOCK));

        while !self.peek(TCode::EOI) {
            let func = self.parse_func();
            global._push(func);
        }

        self.indent_dec();
        global
    }

    // func = FUNC ID PAREN_L [ ID { COMMA ID } ] PAREN_R block
    pub fn parse_func(&mut self) -> MTree {
        self.indent_print("parse_func()");
        self.indent_inc();

        self.expect(TCode::FUNC);
        let id = self.curr();
        self.expect(TCode::ID(String::new()));

        let mut func_tree = MTree::new(Token::from(TCode::FUNC));
        func_tree._push(MTree::new(Token::from(id)));

        let params = self.parse_params();
        func_tree._push(params);

        let body = self.parse_block();
        func_tree._push(body);

        self.indent_dec();
        func_tree
    }

    // params = PAREN_L [ ID { COMMA ID } ] PAREN_R
    fn parse_params(&mut self) -> MTree {
        self.indent_print("parse_params()");
        self.indent_inc();

        let mut params_tree = MTree::new(Token::from(TCode::PARAMS));
        self.expect(TCode::PAREN_L);

        if self.accept(TCode::PAREN_R) {
            self.indent_dec();
            return params_tree;
        }

        loop {
            let param_id = self.curr();
            self.expect(TCode::ID(String::new()));
            params_tree._push(MTree::new(Token::from(param_id)));
            if !self.accept(TCode::COMMA) {
                break;
            }
        }

        self.expect(TCode::PAREN_R);
        self.indent_dec();
        params_tree
    }

    // block = BRACE_L { stmt } BRACE_R
    fn parse_block(&mut self) -> MTree {
        self.indent_print("parse_block()");
        self.indent_inc();

        let mut block = MTree::new(Token::from(TCode::BLOCK));
        self.expect(TCode::BRACE_L);

        while !self.peek(TCode::BRACE_R) && !self.peek(TCode::EOI) {
            let stmt = self.parse_stmt();
            block._push(stmt);
        }

        self.expect(TCode::BRACE_R);
        self.indent_dec();
        block
    }

    // stmt = let_stmt | assign_stmt | return_stmt | if_stmt | write_stmt | call_stmt
    fn parse_stmt(&mut self) -> MTree {
        self.indent_print("parse_stmt()");
        self.indent_inc();

        let tree = match self.curr() {
            TCode::LET    => self.parse_let_stmt(),
            TCode::RETURN => self.parse_return_stmt(),
            TCode::IF     => self.parse_if_stmt(),
            TCode::WHILE  => self.parse_while_stmt(),
            _ if self.peek_id("write") || self.peek_id("print") => self.parse_write_stmt(),
            TCode::ID(_) => {
                let saved_lexer = self.lexer.clone();
                self.advance();
                if self.accept(TCode::ASSIGN) {
                    self.lexer = saved_lexer;
                    self.parse_assign_stmt()
                } else {
                    self.lexer = saved_lexer;
                    self.parse_call_stmt()
                }
            }
            _ => panic!("Invalid statement starting with {:?}", self.curr()),
        };

        self.indent_dec();
        tree
    }

    // let id [ := expr ] ;
    fn parse_let_stmt(&mut self) -> MTree {
        self.indent_print("parse_let_stmt()");
        self.indent_inc();

        self.expect(TCode::LET);
        let id = self.curr();
        self.expect(TCode::ID(String::new()));

        let mut tree = MTree::new(Token::from(TCode::LET));
        tree._push(MTree::new(Token::from(id)));

        if self.accept(TCode::ASSIGN) {
            let expr = self.parse_expr();
            tree._push(expr);
        }

        self.expect(TCode::SEMICOLON);
        self.indent_dec();
        tree
    }

    // id := expr ;
    fn parse_assign_stmt(&mut self) -> MTree {
        self.indent_print("parse_assign_stmt()");
        self.indent_inc();

        let id = self.curr();
        self.expect(TCode::ID(String::new()));
        self.expect(TCode::ASSIGN);

        let mut tree = MTree::new(Token::from(TCode::ASSIGN));
        tree._push(MTree::new(Token::from(id)));
        tree._push(self.parse_expr());

        self.expect(TCode::SEMICOLON);
        self.indent_dec();
        tree

    }

    // return expr ;
    fn parse_return_stmt(&mut self) -> MTree {
        self.indent_print("parse_return_stmt()");
        self.indent_inc();

        self.expect(TCode::RETURN);
        let mut tree = MTree::new(Token::from(TCode::RETURN));
        tree._push(self.parse_expr());
        tree.print();
        self.expect(TCode::SEMICOLON);

        self.indent_dec();
        tree
    }

    // if expr block [ else ( if expr block | block ) ]
    fn parse_if_stmt(&mut self) -> MTree {
        self.indent_print("parse_if_stmt()");
        self.indent_inc();

        self.expect(TCode::IF);
        let cond = self.parse_expr();

        let then_block = self.parse_block();

        let mut tree = MTree::new(Token::from(TCode::IF));
        tree._push(cond);
        tree._push(then_block);

        if self.accept(TCode::ELSE) {
            // tree._push(self.parse_block());
            let else_part = if self.peek(TCode::IF) {
                self.parse_if_stmt()
            } else {
                self.parse_block()
            };
            tree._push(else_part);
        }
        self.indent_dec();
        tree
    }

    fn parse_while_stmt(&mut self) -> MTree {
        self.indent_print("parse_while_stmt()");
        self.indent_inc();

        self.expect(TCode::WHILE);
        let cond = self.parse_expr();

        let body = self.parse_block();

        let mut tree = MTree::new(Token::from(TCode::WHILE));
        tree._push(cond);
        tree._push(body);

        self.indent_dec();
        tree
    }

    // print expr ;
    fn parse_write_stmt(&mut self) -> MTree {
        self.indent_print("parse_write_stmt()");
        self.indent_inc();

        if self.peek_id("write") {
            self.expect_id("write");
        } else {
            self.expect_id("print");
        }

        let arg = self.parse_expr();

        let mut tree = MTree::new(Token::from(TCode::WRITE));
        tree._push(arg);

        self.expect(TCode::SEMICOLON);

        self.indent_dec();
        tree
    }

    // call_stmt = ID PAREN_L [ expr { COMMA expr } ] PAREN_R ;
    fn parse_call_stmt(&mut self) -> MTree {
        self.indent_print("parse_call_stmt()");
        self.indent_inc();

        let call_tree = self.parse_call_expr();
        self.expect(TCode::SEMICOLON);

        self.indent_dec();
        call_tree
    }

    // call_expr = ID PAREN_L [ expr { COMMA expr } ] PAREN_R
    fn parse_call_expr(&mut self) -> MTree {
        let id = self.curr();
        self.expect(TCode::ID(String::new()));
        self.expect(TCode::PAREN_L);

        let mut call_tree = MTree::new(Token::from(TCode::CALL));
        call_tree._push(MTree::new(Token::from(id)));

        if !self.peek(TCode::PAREN_R) {
            loop {
                call_tree._push(self.parse_expr());
                if !self.accept(TCode::COMMA) {
                    break;
                }
            }
        }

        self.expect(TCode::PAREN_R);
        call_tree
    }

    // expr = or_expr
    pub fn parse_expr(&mut self) -> MTree {
        // println!("parse_expr {:?}", self.curr());
        self.parse_or_expr()
    }

    fn parse_or_expr(&mut self) -> MTree {
        // println!("parse_or_expr {:?}", self.curr());
        let mut left = self.parse_and_expr();
        while self.accept(TCode::OR) {
            let mut node = MTree::new(Token::from(TCode::OR));
            node._push(left);
            node._push(self.parse_and_expr());
            left = node;
        }
        left
    }

    fn parse_and_expr(&mut self) -> MTree {
        // println!("parse_and_expr {:?}", self.curr());
        let mut left = self.parse_rel_expr();
        while self.accept(TCode::AND) {
            let mut node = MTree::new(Token::from(TCode::AND));
            node._push(left);
            node._push(self.parse_rel_expr());
            left = node;
        }
        left
    }

    fn parse_rel_expr(&mut self) -> MTree {
        // println!("parse_rel_expr {:?}", self.curr());
        let mut left = self.parse_add_expr();
        while matches!(self.curr(), TCode::LT | TCode::GT | TCode::EQ | TCode::NOT_EQ) {
            let op = self.curr();
            self.advance();
            let mut node = MTree::new(Token::from(op));
            node._push(left);
            node._push(self.parse_add_expr());
            left = node;
        }
        left
    }

    fn parse_add_expr(&mut self) -> MTree {
        // println!("parse_add_expr {:?}", self.curr());
        let mut left = self.parse_mul_expr();
        while matches!(self.curr(), TCode::ADD | TCode::SUB) {
            let op = self.curr();
            self.advance();
            let mut node = MTree::new(Token::from(op));
            node._push(left);
            node._push(self.parse_mul_expr());
            left = node;
        }
        left
    }

    fn parse_mul_expr(&mut self) -> MTree {
        // println!("parse_mul_expr {:?}", self.curr());
        let mut left = self.parse_unary_expr();
        while matches!(self.curr(), TCode::MULT | TCode::DIV) {
            let op = self.curr();
            self.advance();
            let mut node = MTree::new(Token::from(op));
            node._push(left);
            node._push(self.parse_unary_expr());
            left = node;
        }
        left
    }

    fn parse_unary_expr(&mut self) -> MTree {
        // println!("parse_unary_expr {:?}", self.curr());
        if matches!(self.curr(), TCode::NOT | TCode::SUB) {
            let op = self.curr();
            self.advance();
            let mut node = MTree::new(Token::from(op));
            node._push(self.parse_unary_expr());
            node
        } else {
            self.parse_primary()
        }
    }

    fn parse_primary(&mut self) -> MTree {
        // println!("parse_primary {:?}", self.curr());
        match self.curr() {
            TCode::ID(_) => {
                let saved = self.lexer.clone();
                let id = self.curr();
                self.advance();

                if self.peek(TCode::PAREN_L) {
                    self.lexer = saved;
                    self.parse_call_expr()
                } else {
                    // let id = self.curr();
                    // self.advance();
                    MTree::new(Token::from(id))
                }
            }

            TCode::VAL(_) => {
                let val = self.curr();
                self.advance();
                MTree::new(Token::from(val))
            }

            TCode::PAREN_L => {
                self.advance();
                let expr = self.parse_expr();
                self.expect(TCode::PAREN_R);
                expr
            }

            _ => panic!("Unexpected primary: {:?}", self.curr()),
        }
    }
}