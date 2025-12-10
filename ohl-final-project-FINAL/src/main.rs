#![allow(non_snake_case)]

use std::{env, fs};
use std::fs::read_to_string;
use std::ops::Deref;
use std::rc::Rc;
use crate::analyzer::Analyzer;
use crate::evaluator::Evaluator;
use crate::hw_assignment_3::Lexer;
use crate::mtree::MTree;
use crate::token::{Token, TCode};
use crate::value::{DValue};
use crate::hw_assignment_3::Lexer;
use crate::hw_assignment_4::Parser;

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
mod hw_assignment_4;

// ======== COMMAND LINE PRINT FUNCTIONS. ========
fn print_help_for(command: &str) {
    match command {
        "help" => {
            println!("help [command]: \n- prints help info for commands\n");
        }
        "print" => {
            println!("print <file> [--numbered]: \n- prints out the contents of a file to the output. If numbered is true, then it will list every line with a number before it\n");
        }
        "list" => {
            println!("list commands OR list tokens: \n- prints the list of all commands with or without the second field being entered as [commands]\n- if the 2nd field is [tokens], will print out all tokens in the token enum\n");
        }
        "tokenize" => {
            println!("tokenize <file>: \n- tokenizes the input from a file and then prints out the token form of the function\n");
        }
        "parse" => {
            println!("parse <file>: \n- tokenizes & parses the input from a file and then prints out resulting parse tree\n");
        }
        "execute" => {
            println!("execute <file>: \n-tokenizes & parses & analyzes & executes the input from the file and prints out resulting value.\n");
        }
        "example" => {
            println!("example <\"OHL\" | \"YARRICK\">: \n-prints one of two examples that utilize the analyzer and executor on a predefined tree.");
        }
        _ => {
            println!("Unknown command: {}.\n", command);
        }
    }
}

fn ohl_analyzer_evaluator_sample_function() {

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
}

fn yarrick_analyzer_evaluator_sample_function() {
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

// ======== MAIN. ========
fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let valid_commands: Vec<String> = vec![
        String::from("help"),
        String::from("print"),
        String::from("list"),
        String::from("tokenize"),
        String::from("parse"),
    ];
    match args[0].as_str() {
        "help" => {
            if args.len() > 1 {
                let cmd = &args[1];
                if !valid_commands.contains(&cmd) {
                    println!("INVALID HELP COMMAND");
                }
                else {
                    println!("Help Info For {}", cmd);
                    print_help_for(cmd);
                }
            }
            else {
                println!("All Command Help Info:");
                for string in valid_commands {
                    print_help_for(string.as_str());
                }
            }
        }
        "print" => {
            if args.len() < 2 {
                println!("Error: 'print' command requires a file path.\n Usage: print <file> [--numbered]");
                return;
            }
            let numbered = args.len() > 2 && args[2] == "--numbered";
            let file_path = &args[1];
            let contents = fs::read_to_string(file_path).expect("Should have been able to read the file");
            println!("Printing contents of {} :", file_path);
            if numbered {
                for (i, line) in contents.lines().enumerate() {
                    println!("{}: {}", i + 1, line);
                }
            }
            else {
                println!("{}", contents);
            }
        }
        "list" => {
            let print_all_string = String::from("All commands:\n\t help \n\t print \n\t list [commands] \n\t list tokens \n\t example \n\t tokenize \n\t parse \n\t execute \n\t example");
            if args.len() < 2 {
                println!("{}", print_all_string);
            }
            else {
                let cmd = &args[1];
                match cmd.as_str() {
                    "commands" => {
                        println!("{}", print_all_string);
                    }
                    "tokens" => {
                        println!("====Tokens====\n
                        General: EOI, ERROR\n
                        Id and Value atoms: ID(String), VAL(DValue)\n
                        Assignment operator: ASSIGN\n
                        Logical operators: NOT, AND, OR\n
                        Relational operators: LT, GT, EQ, NOT_EQ\n
                        Arithmetic operators: ADD, SUB, MULT, DIV\n
                        Nesting: PAREN_L, PAREN_R, BRACE_L, BRACE_R,\n
                        Separators: COMMA, SEMICOLON\n
                        Keywords: FUNC, LET, IF, ELSE, WHILE, RETURN, READ, WRITE,\n
                        Meta-tokens: BLOCK, PARAMS, CALL\n");
                    }
                    _ => {
                        println!("{}", print_all_string);
                    }
                }
            }
        }
        "tokenize" => {
            if args.len() < 2 {
                println!("Error: 'tokenize' requires a file path.\n Usage: tokenize <file name>");
                return;
            }
            let file_path = &args[1];
            println!("Running tokenization of file {}: ", file_path);
            let mut lexer = Lexer::new();

            lexer.set_input(file_path.clone());
            lexer.print_tokens();
        }
        "parse" => {
            if args.len() < 2 {
                println!("Error: 'parse' requires a file path.\n Usage: parse <file name>");
                return;
            }
            let file_path = &args[1];
            println!("Running parser to tokenize & parse a file {}:", file_path);
            // create recursive descent parser
            let mut lexer = Lexer::new();
            lexer.set_input(file_path.clone()); // Using the testingparser.txt file for this one.
            let mut parser = Parser::new(lexer);
            //parser.lexer.advance();
            // start recursive descent parsing
            let tree = parser.analyze();

            println!("\nMTree:");
            tree.print();
        }
        "execute" => {
            println!("Execute given file.");
        }
        "example" => {
            if args.len() < 2 {
                println!("Error: 'example' requires a name specification.\n Usage: example [OHL | YARRICK]");
                return;
            }
            let run_type = &args[1];

            if run_type.eq(&String::from("OHL")) {
                println!("Running ohl example to analyze and evaluate a hard-coded tree.");
                ohl_analyzer_evaluator_sample_function();
            }
            else if run_type.eq(&String::from("YARRICK")) {
                println!("Running yarrick example to analyze and evaluate a hard-coded tree.");
                yarrick_analyzer_evaluator_sample_function();
            }
            else {
                println!("Error: 'example' requires either \"OHL\" or \"YARRICK\" as a second flag.");
                return;
            }
        }
        _ => {
            println!("Unknown command: {}.", args[0]);
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