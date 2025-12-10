#![allow(non_snake_case)]

use std::fs::read_to_string;
use std::ops::Deref;
use std::rc::Rc;
use crate::analyzer::Analyzer;
use crate::evaluator::Evaluator;
use crate::hw_assignment_3::Lexer;
use crate::mtree::MTree;
use crate::token::{Token, TCode};
use crate::value::{DValue};

mod token;
mod mtree;
mod log;
mod analyzer;
mod typ;
mod value;
mod frame_analyze;
mod evaluator;
mod frame_call;
mod hw_assignment_3;

fn main() {

    // --------------------------------------------------------
    // --------------------------------------------------------
    //
    // func fac(n)
    // [
    //     write n;
    //     if n < 2 [
    //         return 1;
    //     ] else [
    //         return n * fac(n - 1);
    //     ]
    // ]
    //
    // func main()
    // [
    //     n = fac(3);
    //     write n;
    // ]

    // --------------------------------------------------------
    // build tree of func "fac"
    // --------------------------------------------------------

    PrintFromFile();
    return;

    let mtree_fac_base = MTree::new( Token::from( TCode::VAL(DValue::I64(1))));

    let mtree_fac_recursive = MTree {
        token: Token::from( TCode::MULT),
        children: vec![
            Rc::new(MTree::new(Token::id("n")) ),
            Rc::new( MTree {
                token: Token::from( TCode::CALL),
                children: vec![
                    Rc::new(MTree::new(Token::id("fac")) ),
                    Rc::new( MTree {
                        token: Token::from( TCode::SUB),
                        children: vec![
                            Rc::new(MTree::new(Token::id("n")) ),
                            Rc::new(MTree::new(
                                Token::from( TCode::VAL(DValue::I64(1)))
                            )),
                        ]
                    })
                ]
            })
        ]
    };

    let mtree_fac_block = MTree {
        token: Token::from( TCode::BLOCK),
        children: vec![
            Rc::new( MTree {
                token: Token::from(TCode::WRITE),
                children: vec![
                    Rc::new(MTree::new(Token::id("n")) )
                ]
            }),
            Rc::new( MTree {
                token: Token::from( TCode::IF),
                children: vec![
                    Rc::new(MTree {
                        token: Token::from(TCode::LT),
                        children: vec![
                            Rc::new(MTree::new( Token::id("n") )),
                            Rc::new(MTree::new(
                                Token::from( TCode::VAL(DValue::I64(2)))
                            ))
                        ]
                    }),
                    Rc::new(MTree {
                        token: Token::from(TCode::BLOCK),
                        children: vec![
                            Rc::new(MTree {
                                token: Token::from(TCode::RETURN),
                                children: vec![ Rc::new(mtree_fac_base) ]
                            }),
                        ]
                    }),
                    Rc::new(MTree {
                        token: Token::from(TCode::BLOCK),
                        children: vec![
                            Rc::new(MTree {
                                token: Token::from(TCode::RETURN),
                                children: vec![ Rc::new(mtree_fac_recursive) ]
                            }),
                        ]
                    })
                ]
            }),
        ]
    };

    // let global = Rc::new(RefCell::new(Frame::new()));
    let rc_mtree_fac = Rc::new(MTree {
        token: Token::from(TCode::FUNC),
        children: vec![
            Rc::new( MTree {
                token: Token::id("fac"),
                children: vec![]
            }),
            Rc::new( MTree {
                token: Token::from( TCode::PARAMS),
                children: vec![ // one parameter "n"
                    Rc::new( MTree::new( Token::id("n")))
                ]
            }),
            Rc::new(mtree_fac_block)
        ]
    });

    // --------------------------------------------------------
    // build tree of func "main"
    // --------------------------------------------------------

    let mtree_main_block = MTree {
        token: Token::from( TCode::BLOCK),
        children: vec![
            Rc::new( MTree {
                token: Token::from(TCode::ASSIGN),
                children: vec![
                    Rc::new(MTree::new(Token::id("n")) ),
                    Rc::new( MTree {
                        token: Token::from( TCode::CALL),
                        children: vec![
                            Rc::new(MTree::new(Token::id("fac")) ),
                            Rc::new(MTree::new(
                                Token::from( TCode::VAL(DValue::I64(5)))
                            )),
                        ]
                    })
                ]
            }),
            Rc::new( MTree {
                token: Token::from(TCode::WRITE),
                children: vec![
                    Rc::new(MTree::new(Token::id("n")) )
                ]
            }),
        ]
    };

    let rc_mtree_main = Rc::new(MTree {
        token: Token::from(TCode::FUNC),
        children: vec![
            Rc::new(MTree::new(Token::id("main"))),
            Rc::new(MTree {
                token: Token::from( TCode::PARAMS),
                children: vec![] // no parameters
            }),
            Rc::new(mtree_main_block)
        ]
    });


    // --------------------------------------------------------
    // build tree global block
    // --------------------------------------------------------

    let rc_mtree_global = Rc::new(MTree {
        token: Token::from( TCode::BLOCK),
        children: vec![
            rc_mtree_fac.clone(),
            rc_mtree_main.clone(),
            Rc::new( MTree {
                token: Token::from( TCode::CALL),
                children: vec![
                    Rc::new(MTree::new(Token::id("main")) ),
                ]
            })
        ]
    });

    println!("----------------------------------------------------------------");
    println!("\nMTree (Parsed) 'global':\n");
    rc_mtree_global.print();


    // --------------------------------------------------------
    // analyze tree
    // --------------------------------------------------------
    println!("----------------------------------------------------------------");
    let analyzer = Analyzer::new();
    let rc_tree_analyzed = analyzer.analyze_global(rc_mtree_global.clone());
    println!("\nMTree (Analyzed) 'global':\n");
    rc_tree_analyzed.print();


    // --------------------------------------------------------
    // evaluate tree
    // --------------------------------------------------------

    println!("----------------------------------------------------------------");
    println!("\nEVALUATE MTree (Analyzed) 'global' :\n");
    let mut evaluator = Evaluator::new();
    evaluator.evaluate(rc_tree_analyzed.deref());

    /*
        --------------------------------------------------------
        EXAMPLE PROGRAM - 2.
        --------------------------------------------------------

        func main() {
            let n = 0;

            while n < 10 {
                n = n + 1;
            }

            write n;
        }
     */

    let mtree_fac_base = MTree::new( Token::from( TCode::VAL(DValue::I64(1))));

    let mtree_main_block = MTree {
        token: Token::from( TCode::BLOCK),
        children: vec![
            Rc::new( MTree {
                token: Token::from(TCode::ASSIGN),
                children: vec![
                    Rc::new(MTree::new(Token::id("n")) ),
                    Rc::new( MTree::new(Token::from(TCode::VAL(DValue::I64(0))))),
                ]
            }),

            Rc::new(MTree {
                token: Token::from(TCode::WHILE),
                children: vec! [
                    Rc::new(MTree {
                        token: Token::from(TCode::LT),
                        children: vec! [
                            Rc::new(MTree::new( Token::id("n") )),
                            Rc::new(MTree::new(
                                Token::from( TCode::VAL(DValue::I64(10)))
                            ))
                        ]
                    }),
                    Rc::new(MTree {
                        token: Token::from(TCode::BLOCK),
                        children: vec! [
                            Rc::new(MTree {
                                token: Token::from(TCode::ASSIGN),
                                children: vec! [
                                    Rc::new(MTree::new(Token::id("n"))),
                                    Rc::new(MTree {
                                        token: Token::from(TCode::ADD),
                                        children: vec! [
                                            Rc::new(MTree::new( Token::id("n") )),
                                            Rc::new(MTree::new(
                                                Token::from( TCode::VAL(DValue::I64(1)))
                                            ))
                                        ]
                                    })
                                ]
                            }),
                            Rc::new(MTree{
                                token: Token::from(TCode::WRITE),
                                children: vec! [
                                    Rc::new(MTree::new(Token::id("n"))),
                                ]
                            })
                        ]
                    })
                ]
            }),

            Rc::new( MTree {
                token: Token::from(TCode::WRITE),
                children: vec![
                    Rc::new(MTree::new(Token::id("n")) )
                ]
            }),
        ]
    };

    let rc_mtree_main = Rc::new(MTree {
        token: Token::from(TCode::FUNC),
        children: vec![
            Rc::new(MTree::new(Token::id("main"))),
            Rc::new(MTree {
                token: Token::from( TCode::PARAMS),
                children: vec![] // no parameters
            }),
            Rc::new(mtree_main_block)
        ]
    });

    let rc_mtree_global = Rc::new(MTree {
        token: Token::from( TCode::BLOCK),
        children: vec![
            // rc_mtree_fac.clone(),
            rc_mtree_main.clone(),
            Rc::new( MTree {
                token: Token::from( TCode::CALL),
                children: vec![
                    Rc::new(MTree::new(Token::id("main")) ),
                ]
            })
        ]
    });

    println!("----------------------------------------------------------------");
    println!("\nMTree (Parsed) 'global':\n");
    rc_mtree_global.print();

    // --------------------------------------------------------
    // analyze tree
    // --------------------------------------------------------
    println!("----------------------------------------------------------------");
    let analyzer = Analyzer::new();
    let rc_tree_analyzed = analyzer.analyze_global(rc_mtree_global.clone());
    println!("\nMTree (Analyzed) 'global':\n");
    rc_tree_analyzed.print();


    // --------------------------------------------------------
    // evaluate tree
    // --------------------------------------------------------

    println!("----------------------------------------------------------------");
    println!("\nEVALUATE MTree (Analyzed) 'global' :\n");
    let mut evaluator = Evaluator::new();
    evaluator.evaluate(rc_tree_analyzed.deref());
}

fn PrintFromFile() {
    let mut lex = Lexer::new();
    lex.set_input(String::from("example.txt"));

    lex.advance();
    lex.print_token();

    while lex.token.clone().unwrap() != TCode::EOI {
        lex.advance();
        lex.print_token();
    }
}