#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(non_snake_case)]

use crate::hw_assignment_3::*;
use crate::token::*;
use crate::value::DValue;
use crate::mtree::*;

const INDENT : usize = 2;

pub struct Parser {
    pub(crate) lexer: Lexer,
    pub(crate) indent: usize,
}

impl Parser {

    pub fn new(mut lexer: Lexer) -> Parser {
        lexer.advance();
        Parser {
            lexer,
            indent: 0,
        }
    }
    pub fn analyze(&mut self) -> MTree {
        self.indent = 0;
        let tree = self.parse_program();
        self.expect(TCode::EOI);
        tree
    }
}

impl Parser { // utility functions for lexer

    pub fn curr(&self) -> TCode {
        self.lexer.curr()
    }
    pub fn advance(&mut self) {
        self.lexer.advance();
    }
    pub fn peek(&self, symbol: TCode) -> bool {
        let mut lexer = self.lexer.clone();
        while lexer.curr() == TCode::LINEBREAK { lexer.advance(); }
        lexer.curr() == symbol
    }
    pub fn expect(&mut self, symbol: TCode) {
        while self.curr() == TCode::LINEBREAK { self.advance(); }
        if self.curr() == symbol {
            self.advance();
            println!("{:<indent$}expect({symbol:?})", "", indent = self.indent);
        }
        else {
            panic!("Did not expect '{symbol:?}'!");
        }
    }
    pub fn accept(&mut self, symbol: TCode) -> bool {
        if self.curr() == symbol {
            self.advance();
            true
        }
        else {
            false
        }
    }
    pub fn skip_whitespace(&mut self) {
        while self.curr() == TCode::LINEBREAK {
            self.advance();
        }
    }
}

impl Parser { // utility functions for pretty print

    pub(crate) fn indent_print(&mut self, msg: &'static str) {
        println!("{:<indent$}{:}", "", msg, indent=self.indent);
    }
    pub(crate) fn indent_increment(&mut self) {
        self.indent += INDENT;
    }
    pub(crate) fn indent_decrement(&mut self) {
        self.indent -= INDENT;
    }
}

impl Parser {  // simple recursive descent parser

    pub fn parse(&mut self) -> MTree {
        self.parse_program()
    }
    // EBNF: program = func
    pub fn parse_program(&mut self) -> MTree {
        self.indent_print("parse_program()");
        self.indent_increment();
        let mut tree = MTree::new(TCode::META_PROGRAM);
        while !self.peek(TCode::EOI) {
            self.skip_whitespace();
            let func = self.parse_func();
            tree._push(func);
        }
        self.indent_decrement();
        tree
    }

    // EBNF: func = FUNC ID param_list block
    pub fn parse_func(&mut self) -> MTree {
        self.indent_print("parse_func()");
        self.indent_increment();
        let mut tree = MTree::new(TCode::META_FUNC);
        self.expect(TCode::FUNC);
        let id = self.curr();
        self.expect(TCode::ID(String::new()));
        tree._push(MTree::new(id));
        let params = self.parse_parameter_list();
        tree._push(params);
        let block = self.parse_block_nest();
        tree._push(block);
        self.indent_decrement();
        tree
    }

    // EBNF: param_list = PARENS_L (param (COMMA param) ) PARENS_R
    pub fn parse_parameter_list(&mut self) -> MTree {
        self.indent_print("parse_parameter_list()");
        self.indent_increment();
        let mut tree = MTree::new(TCode::META_PARAM_LIST);
        self.expect(TCode::PAREN_L);
        if self.accept(TCode::PAREN_R) {
            self.indent_decrement();
            return tree;
        }
        let param = self.parse_parameter();
        tree._push(param);
        while self.accept(TCode::COMMA) {   // list -> ( {param{,param}+}? )
            let param = self.parse_parameter(); // param -> id : id
            tree._push(param);
        }
        self.expect(TCode::PAREN_R);
        self.indent_decrement();
        tree
    }

    // EBNF: param = ID
    pub fn parse_parameter(&mut self) -> MTree {
        self.indent_print("parse_parameter()");
        self.indent_increment();
        let mut tree = MTree::new(TCode::META_PARAM);
        let id = self.curr();
        self.expect(TCode::ID(String::new()));
        tree._push(MTree::new(id));
        self.indent_decrement();
        tree
    }

    // EBNF: block = BRACE_L statement BRACE_R
    pub fn parse_block_nest(&mut self) -> MTree {
        self.indent_print("parse_block_nest()");
        self.indent_increment();
        let mut tree = MTree::new(TCode::META_BLOCK);
        self.expect(TCode::BRACE_L);
        while !self.peek(TCode::BRACE_R) {
            self.skip_whitespace();
            let stmt = self.parse_stmt();
            tree._push(stmt);
        }
        self.expect(TCode::BRACE_R);
        self.indent_decrement();
        tree
    }

    pub fn parse_block_list(&mut self) -> MTree {
        self.indent_print("parse_block_list()");
        self.indent_increment();
        let block = self.parse_block_nest();
        self.indent_decrement();
        block
    }

    // EBNF: statement = let_statement | return_statement | if_statement | assign_statement | print_statement
    pub fn parse_stmt(&mut self) -> MTree {
        self.indent_print("parse_statement()");
        self.indent_increment();
        self.skip_whitespace();
        let tree = if self.peek(TCode::LET) {
            self.parse_let_stmt()
        } else if self.peek(TCode::IF) {
            self.parse_if_stmt()
        } else if self.peek(TCode::RETURN) {
            self.parse_return_stmt()
        } else if self.peek(TCode::WRITE) {
            self.parse_print_stmt()
        } else {
            panic!("Unexpected statement {:?}", self.curr());
        };
        self.indent_decrement();
        tree
    }

    // EBNF: let_stmt = LET ID (COLON type) ASSIGN expr SEMICOLON
    pub fn parse_let_stmt(&mut self) -> MTree {
        self.indent_print("parse_let()");
        self.indent_increment();
        let mut tree = MTree::new(TCode::META_LET);
        self.expect(TCode::LET);
        let id = self.curr();
        self.expect(TCode::ID(String::new()));
        tree._push(MTree::new(id));
        let typ;
        if self.accept(TCode::COLON) {
            typ = self.parse_type();
        } else {
            typ = MTree::new(TCode::META_INFER);
        }
        tree._push(typ);
        self.expect(TCode::ASSIGN);
        let expr = self.parse_expr();
        tree._push(expr);
        self.expect(TCode::SEMICOLON);
        self.indent_decrement();
        tree
    }

    // EBNF: assign_stmt = ID (ASSIGN | ADD_ASSIGN | ...) expr SEMICOLON
    pub fn parse_assign_stmt(&mut self) -> MTree {
        self.indent_print("parse_assign()");
        self.indent_increment();
        let mut tree = MTree::new(TCode::META_ASSIGN);
        let id = self.curr();
        self.expect(TCode::ID(String::new()));
        tree._push(MTree::new(id));
        let op = self.curr();
        if matches!(op, TCode::ASSIGN | TCode::ADD | TCode::SUB | TCode::MULT | TCode::DIV) {
            self.advance();
            tree.TCode = op;  // Use the specific op as root TCode
        } else {
            panic!("Expected assignment operator, found {:?}", op);
        }
        let expr = self.parse_expr();
        tree._push(expr);
        self.expect(TCode::SEMICOLON);
        self.indent_decrement();
        tree
    }

    // EBNF: print_stmt = PRINT expr (COMMA expr)* SEMICOLON
    pub fn parse_print_stmt(&mut self) -> MTree {
        self.indent_print("parse_print()");
        self.indent_increment();
        let mut tree = MTree::new(TCode::META_PRINT);
        self.expect(TCode::PRINT);
        loop {
            let expr = self.parse_expr();
            tree._push(expr);
            if !self.accept(TCode::COMMA) {
                break;
            }
        }
        self.expect(TCode::SEMICOLON);
        self.indent_decrement();
        tree
    }

    // EBNF: return_statement = RETURN expr SEMICOLON
    pub fn parse_return_stmt(&mut self) -> MTree {
        self.indent_print("parse_return()");
        self.indent_increment();
        let mut tree = MTree::new(TCode::META_RETURN);
        self.expect(TCode::RETURN);
        let expr = self.parse_expr();
        tree._push(expr);
        self.expect(TCode::SEMICOLON);
        self.indent_decrement();
        tree
    }

    // EBNF: if_statement = IF expr block (ELSE else_part)
    pub fn parse_if_stmt(&mut self) -> MTree {
        self.indent_print("parse_if()");
        self.indent_increment();
        let mut tree = MTree::new(TCode::META_IF);
        self.expect(TCode::IF);
        let cond = self.parse_expr();
        tree._push(cond);
        let then_block = self.parse_block_nest();
        tree._push(then_block);
        if self.accept(TCode::ELSE) {
            let else_part = self.parse_else_part();
            tree._push(else_part);
        }
        self.indent_decrement();
        tree
    }

    // EBNF: else_part = IF expr block (ELSE else_part) | block
    pub fn parse_else_part(&mut self) -> MTree {
        self.indent_print("parse_else_part()");
        self.indent_increment();
        let tree;
        if self.peek(TCode::IF) {
            self.expect(TCode::IF);
            let mut elseif_tree = MTree::new(TCode::META_ELSE_IF);
            let cond = self.parse_expr();
            elseif_tree._push(cond);
            let then_block = self.parse_block_nest();
            elseif_tree._push(then_block);
            if self.accept(TCode::ELSE) {
                let next_else = self.parse_else_part();
                elseif_tree._push(next_else);
            }
            tree = elseif_tree;
        } else {
            tree = self.parse_block_nest();
        }
        self.indent_decrement();
        tree
    }

    // EBNF: expr = or_expr
    pub fn parse_expr(&mut self) -> MTree {
        self.indent_print("parse_expr()");
        self.indent_increment();
        let expr = self.parse_or_expr();
        self.indent_decrement();
        expr
    }

    // EBNF: or_expr = and_expr ( OR and_expr )*
    fn parse_or_expr(&mut self) -> MTree {
        let mut left = self.parse_and_expr();
        while self.accept(TCode::OR) {
            let mut bin_tree = MTree::new(TCode::OR);
            bin_tree._push(left);
            let right = self.parse_and_expr();
            bin_tree._push(right);
            left = bin_tree;
        }
        left
    }

    // EBNF: and_expr = rel_expr ( AND rel_expr )*
    fn parse_and_expr(&mut self) -> MTree {
        let mut left = self.parse_rel_expr();
        while self.accept(TCode::AND) {
            let mut bin_tree = MTree::new(TCode::AND);
            bin_tree._push(left);
            let right = self.parse_rel_expr();
            bin_tree._push(right);
            left = bin_tree;
        }
        left
    }

    // EBNF: rel_expr = add_expr ( rel_op add_expr )*
    fn parse_rel_expr(&mut self) -> MTree {
        let mut left = self.parse_add_expr();
        while matches!(self.curr(), TCode::EQ | TCode::LT | TCode::GT | TCode::NOT_EQ) {
            let op = self.curr();
            self.advance();
            let mut bin_tree = MTree::new(op);
            bin_tree._push(left);
            let right = self.parse_add_expr();
            bin_tree._push(right);
            left = bin_tree;
        }
        left
    }

    // EBNF: add_expr = mul_expr ( (ADD | SUB) mul_expr )*
    fn parse_add_expr(&mut self) -> MTree {
        let mut left = self.parse_mul_expr();
        while matches!(self.curr(), TCode::ADD | TCode::SUB) {
            let op = self.curr();
            self.advance();
            let mut bin_tree = MTree::new(op);
            bin_tree._push(left);
            let right = self.parse_mul_expr();
            bin_tree._push(right);
            left = bin_tree;
        }
        left
    }

    // EBNF: mul_expr = unary_expr ( (MUL | DIV) unary_expr )*
    fn parse_mul_expr(&mut self) -> MTree {
        let mut left = self.parse_unary_expr();
        while matches!(self.curr(), TCode::MULT | TCode::DIV) {
            let op = self.curr();
            self.advance();
            let mut bin_tree = MTree::new(op);
            bin_tree._push(left);
            let right = self.parse_unary_expr();
            bin_tree._push(right);
            left = bin_tree;
        }
        left
    }


    // EBNF: unary_expr = (NOT | SUB) unary_expr | primary
    fn parse_unary_expr(&mut self) -> MTree {
        if matches!(self.curr(), TCode::NOT | TCode::SUB) {
            let op = self.curr();
            self.advance();
            let mut tree = MTree::new(op);
            tree._push(self.parse_unary_expr());
            tree
        } else {
            self.parse_primary()
        }
    }

    // EBNF: primary = ID | VAL(DValue::I64) | VAL(DValue::BOOL(true) | VAL(DValue::BOOL(false) | PARENS_L expr PARENS_R | ID PARENS_L (expr (COMMA expr)*)? PARENS_R
    fn parse_primary(&mut self) -> MTree {
        self.indent_print("parse_primary()");
        self.indent_increment();
        let TCode = self.curr();
        let mut tree;
        match TCode {
            TCode::ID(ref s) => {
                let id_str = s.clone();
                self.advance();
                if self.peek(TCode::PAREN_L) {
                    // Function call
                    tree = MTree::new(TCode::META_CALL);
                    tree._push(MTree::new(TCode::ID(id_str)));
                    self.expect(TCode::PAREN_L);
                    if !self.peek(TCode::PAREN_R) {
                        loop {
                            let arg = self.parse_expr();
                            tree._push(arg);
                            if !self.accept(TCode::COMMA) {
                                break;
                            }
                        }
                    }
                    self.expect(TCode::PAREN_R);
                } else {
                    tree = MTree::new(TCode::ID(id_str));
                }
            }
            TCode::VAL(DValue::I64(_)) | TCode::VAL(DValue::BOOL(true)) | TCode::VAL(DValue::BOOL(false)) => {
                self.advance();
                tree = MTree::new(TCode);
            }
            TCode::PAREN_L => {
                self.advance();
                tree = self.parse_expr();
                self.expect(TCode::PAREN_R);
            }
            _ => panic!("Unexpected primary {TCode:?}"),
        }
        self.indent_decrement();
        tree
    }
}
