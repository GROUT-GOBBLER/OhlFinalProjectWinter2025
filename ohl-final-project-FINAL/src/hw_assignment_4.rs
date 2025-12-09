#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::mem::discriminant;
use crate::hw_assignment_3::*;
use std::rc::Rc;

const INDENT : usize = 2;

pub struct Parser {
    lexer: Lexer,
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
        self.expect(Token::EOI);
        tree
    }
}

impl Parser { // utility functions for lexer

    pub fn curr(&self) -> Token {
        self.lexer.curr()
    }
    pub fn advance(&mut self) {
        self.lexer.advance();
    }
    pub fn peek(&self, symbol: Token) -> bool {
        let mut lexer = self.lexer.clone();
        while lexer.curr() == Token::LINEBREAK { lexer.advance(); }
        lexer.curr() == symbol
    }
    pub fn expect(&mut self, symbol: Token) {
        while self.curr() == Token::LINEBREAK { self.advance(); }
        if self.curr() == symbol {
            self.advance();
            println!("{:<indent$}expect({symbol:?})", "", indent = self.indent);
        }
        else {
            panic!("Did not expect '{symbol:?}'!");
        }
    }
    pub fn accept(&mut self, symbol: Token) -> bool {
        if self.curr() == symbol {
            self.advance();
            true
        }
        else {
            false
        }
    }
    pub fn skip_whitespace(&mut self) {
        while self.curr() == Token::LINEBREAK {
            self.advance();
        }
    }
    fn peek_compound_assign(&mut self) -> bool {
        let mut lexer = self.lexer.clone();
        while lexer.curr() == Token::LINEBREAK { lexer.advance(); }
        if !matches!(lexer.curr(), Token::ID(_)) { return false; }
        lexer.advance();
        while lexer.curr() == Token::LINEBREAK { lexer.advance(); }
        matches!(lexer.curr(), Token::ASSIGN | Token::ADD_ASSIGN | Token::SUB_ASSIGN | Token::MUL_ASSIGN | Token::DIV_ASSIGN)
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
        let mut tree = MTree::new(Token::META_PROGRAM);
        while !self.peek(Token::EOI) {
            self.skip_whitespace();
            let func = self.parse_func();
            tree._push(func);
        }
        self.indent_decrement();
        tree
    }

    // EBNF: func = FUNC ID param_list ( : ) (ARROW_R type) block
    pub fn parse_func(&mut self) -> MTree {
        self.indent_print("parse_func()");
        self.indent_increment();
        let mut tree = MTree::new(Token::META_FUNC);
        self.expect(Token::FUNC);
        let id = self.curr();
        self.expect(Token::ID(String::new()));
        tree._push(MTree::new(id));
        let params = self.parse_parameter_list();
        tree._push(params);
        if self.accept(Token::ARROW_R) {
            let ret_type = self.parse_type();
            tree._push(ret_type);
        }
        else {
            tree._push(MTree::new(Token::META_VOID));
        }
        let block = self.parse_block_nest();
        tree._push(block);
        self.indent_decrement();
        tree
    }

    // EBNF: param_list = PARENS_L (param (COMMA param) ) PARENS_R
    pub fn parse_parameter_list(&mut self) -> MTree {
        self.indent_print("parse_parameter_list()");
        self.indent_increment();
        let mut tree = MTree::new(Token::META_PARAM_LIST);
        self.expect(Token::PARENS_L);
        if self.accept(Token::PARENS_R) {
            self.indent_decrement();
            return tree;
        }
        let param = self.parse_parameter();
        tree._push(param);
        while self.accept(Token::COMMA) {   // list -> ( {param{,param}+}? )
            let param = self.parse_parameter(); // param -> id : id
            tree._push(param);
        }
        self.expect(Token::PARENS_R);
        self.indent_decrement();
        tree
    }

    // EBNF: param = ID COLON type
    pub fn parse_parameter(&mut self) -> MTree {
        self.indent_print("parse_parameter()");
        self.indent_increment();
        let mut tree = MTree::new(Token::META_PARAM);
        let id = self.curr();
        self.expect(Token::ID(String::new()));
        tree._push(MTree::new(id));
        self.expect(Token::COLON);
        let typ = self.parse_type();
        tree._push(typ);
        self.indent_decrement();
        tree
    }

    // EBNF: type = TYPE_INT32 | TYPE_FLT32 | TYPE_CHAR
    pub fn parse_type(&mut self) -> MTree {
        self.indent_print("parse_type()");
        self.indent_increment();
        let token = self.curr();
        if matches!(token, Token::TYPE_INT32 | Token::TYPE_FLT32 | Token::TYPE_CHAR) {
            self.advance();
            let tree = MTree::new(token);
            self.indent_decrement();
            tree
        }
        else {
            panic!("Expected type, found {token:?}");
        }
    }

    // EBNF: block = BRACKET_L statement BRACKET_R
    pub fn parse_block_nest(&mut self) -> MTree {
        self.indent_print("parse_block_nest()");
        self.indent_increment();
        let mut tree = MTree::new(Token::META_BLOCK);
        self.expect(Token::BRACKET_L);
        while !self.peek(Token::BRACKET_R) {
            self.skip_whitespace();
            let stmt = self.parse_stmt();
            tree._push(stmt);
        }
        self.expect(Token::BRACKET_R);
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

    fn skip_linebreaks(&mut self) {
        while self.curr() == Token::LINEBREAK {
            self.advance();
        }
    }

    // EBNF: statement = let_statement | return_statement | if_statement | assign_statement | print_statement
    pub fn parse_stmt(&mut self) -> MTree {
        self.indent_print("parse_statement()");
        self.indent_increment();
        self.skip_whitespace();
        let tree = if self.peek(Token::LET) {
            self.parse_let_stmt()
        } else if self.peek(Token::IF) {
            self.parse_if_stmt()
        } else if self.peek(Token::RETURN) {
            self.parse_return_stmt()
        } else if self.peek(Token::PRINT) {
            self.parse_print_stmt()
        } else if self.peek_compound_assign() {
            self.parse_assign_stmt()
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
        let mut tree = MTree::new(Token::META_LET);
        self.expect(Token::LET);
        let id = self.curr();
        self.expect(Token::ID(String::new()));
        tree._push(MTree::new(id));
        let typ;
        if self.accept(Token::COLON) {
            typ = self.parse_type();
        } else {
            typ = MTree::new(Token::META_INFER);
        }
        tree._push(typ);
        self.expect(Token::ASSIGN);
        let expr = self.parse_expr();
        tree._push(expr);
        self.expect(Token::SEMICOLON);
        self.indent_decrement();
        tree
    }

    // EBNF: assign_stmt = ID (ASSIGN | ADD_ASSIGN | ...) expr SEMICOLON
    pub fn parse_assign_stmt(&mut self) -> MTree {
        self.indent_print("parse_assign()");
        self.indent_increment();
        let mut tree = MTree::new(Token::META_ASSIGN);
        let id = self.curr();
        self.expect(Token::ID(String::new()));
        tree._push(MTree::new(id));
        let op = self.curr();
        if matches!(op, Token::ASSIGN | Token::ADD_ASSIGN | Token::SUB_ASSIGN | Token::MUL_ASSIGN | Token::DIV_ASSIGN) {
            self.advance();
            tree.token = op;  // Use the specific op as root token
        } else {
            panic!("Expected assignment operator, found {:?}", op);
        }
        let expr = self.parse_expr();
        tree._push(expr);
        self.expect(Token::SEMICOLON);
        self.indent_decrement();
        tree
    }

    // EBNF: print_stmt = PRINT expr (COMMA expr)* SEMICOLON
    pub fn parse_print_stmt(&mut self) -> MTree {
        self.indent_print("parse_print()");
        self.indent_increment();
        let mut tree = MTree::new(Token::META_PRINT);
        self.expect(Token::PRINT);
        loop {
            let expr = self.parse_expr();
            tree._push(expr);
            if !self.accept(Token::COMMA) {
                break;
            }
        }
        self.expect(Token::SEMICOLON);
        self.indent_decrement();
        tree
    }

    // EBNF: return_statement = RETURN expr SEMICOLON
    pub fn parse_return_stmt(&mut self) -> MTree {
        self.indent_print("parse_return()");
        self.indent_increment();
        let mut tree = MTree::new(Token::META_RETURN);
        self.expect(Token::RETURN);
        let expr = self.parse_expr();
        tree._push(expr);
        self.expect(Token::SEMICOLON);
        self.indent_decrement();
        tree
    }

    // EBNF: if_statement = IF expr block (ELSE else_part)
    pub fn parse_if_stmt(&mut self) -> MTree {
        self.indent_print("parse_if()");
        self.indent_increment();
        let mut tree = MTree::new(Token::META_IF);
        self.expect(Token::IF);
        let cond = self.parse_expr();
        tree._push(cond);
        let then_block = self.parse_block_nest();
        tree._push(then_block);
        if self.accept(Token::ELSE) {
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
        if self.peek(Token::IF) {
            self.expect(Token::IF);
            let mut elseif_tree = MTree::new(Token::META_ELSE_IF);
            let cond = self.parse_expr();
            elseif_tree._push(cond);
            let then_block = self.parse_block_nest();
            elseif_tree._push(then_block);
            if self.accept(Token::ELSE) {
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
        while self.accept(Token::OR) {
            let mut bin_tree = MTree::new(Token::OR);
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
        while self.accept(Token::AND) {
            let mut bin_tree = MTree::new(Token::AND);
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
        while matches!(self.curr(), Token::EQ | Token::LT | Token::GT | Token::NEQ | Token::NLT | Token::NGT) {
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
        while matches!(self.curr(), Token::ADD | Token::SUB) {
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
        while matches!(self.curr(), Token::MUL | Token::DIV) {
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
        if matches!(self.curr(), Token::NOT | Token::SUB) {
            let op = self.curr();
            self.advance();
            let mut tree = MTree::new(op);
            tree._push(self.parse_unary_expr());
            tree
        } else {
            self.parse_primary()
        }
    }

    // EBNF: primary = ID | LIT_INT32 | LIT_FLT32 | TRUE | FALSE | PARENS_L expr PARENS_R | ID PARENS_L (expr (COMMA expr)*)? PARENS_R
    fn parse_primary(&mut self) -> MTree {
        self.indent_print("parse_primary()");
        self.indent_increment();
        let token = self.curr();
        let mut tree;
        match token {
            Token::ID(ref s) => {
                let id_str = s.clone();
                self.advance();
                if self.peek(Token::PARENS_L) {
                    // Function call
                    tree = MTree::new(Token::META_CALL);
                    tree._push(MTree::new(Token::ID(id_str)));
                    self.expect(Token::PARENS_L);
                    if !self.peek(Token::PARENS_R) {
                        loop {
                            let arg = self.parse_expr();
                            tree._push(arg);
                            if !self.accept(Token::COMMA) {
                                break;
                            }
                        }
                    }
                    self.expect(Token::PARENS_R);
                } else {
                    tree = MTree::new(Token::ID(id_str));
                }
            }
            Token::LIT_INT32(_) | Token::LIT_FLT32(_) | Token::LIT_CHAR(_) | Token::TRUE | Token::FALSE | Token::LIT_STRING(_) => {
                self.advance();
                tree = MTree::new(token);
            }
            Token::PARENS_L => {
                self.advance();
                tree = self.parse_expr();
                self.expect(Token::PARENS_R);
            }
            _ => panic!("Unexpected primary {token:?}"),
        }
        self.indent_decrement();
        tree
    }
}
#[derive(Debug, Clone)]
pub struct MTree {
    pub token : Token,
    pub children : Vec<Rc<MTree>>
}

impl MTree {
    pub fn new(token : Token) -> MTree {
        MTree {
            token,
            children : vec![]
        }
    }

    pub fn _push(&mut self, tree : MTree) {
        self.children.push(Rc::new(tree));
    }
    pub fn node_string(&self) -> String {
        format!("{:?}", self.token)
    }
    fn print_recursively(&self, level : usize) {
        let shift = 2*level;
        print!("{:1$}", "", shift);
        println!("{}", self.node_string());
        for child in &self.children {
            child.as_ref().print_recursively(level+1);
        }
    }
    pub fn print(&self) {
        self.print_recursively(0);
    }
}
