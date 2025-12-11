#![allow(non_snake_case)]

use std::env::args;
use std::fs::read_to_string;
use std::ops::Deref;
use std::rc::Rc;
use crate::analyzer::Analyzer;
use crate::evaluator::Evaluator;
use crate::mtree::MTree;
use crate::token::{Token, TCode};
use crate::value::{DValue};
use crate::hw_assignment_3::Lexer;
use crate::hw_assignment_4::Parser;
use std::io::{self, Write};

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

fn main() {
    println!("Type a command (help, print, list, tokenize, parse, execute or exit):");

    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        let mut input = String::new();

        if io::stdin().read_line(&mut input).is_err() {
            println!("Failed to read input.");
            continue;
        }

        let collected: Vec<&str> = input.trim().split_whitespace().collect();
        if collected.is_empty() {
            continue;
        }

        let cmd = collected[0].to_lowercase();
        let args = &collected[1..];

        match cmd.as_str() {
            "exit" => break,
            "help" => {
                if !args.is_empty() {
                    print_help_for(args[0]);
                } else { list_command("commands"); }
            },
            "list" => {
                if !args.is_empty() {
                    list_command(args[0]);
                } else { list_command("commands"); }
            },
            "print" => {
                if args.len() == 0 {
                    println!("File must be given")
                }
                else if args.len() == 1 {
                    let file_path = args[0];
                    print(String::from(file_path), String::from(""));
                }
                else if args.len() == 2 {
                    let file_path = args[0];
                    let flag = args[1];
                    print(String::from(file_path), String::from(flag));
                }
                else {
                    println!("Too many arguments!");
                }
            }
            "tokenize" => {
                if args.is_empty() { println!("File must be given") }
                let file_path = args[0];
                println!("Running tokenization of file {}: ", file_path);

                let mut lexer = Lexer::new();
                lexer.set_input(file_path.clone().parse().unwrap());
                RunLexerOnFile(&mut lexer);
            },
            "parse" => {
                if args.is_empty() { println!("File must be given") }
                let file_path = &args[0];
                println!("Running parser to tokenize & parse a file {}:", file_path);

                // create recursive descent parser
                let mut lexer = Lexer::new();
                lexer.set_input(String::from(file_path.clone()));
                let mut parser = Parser::new(lexer);

                // start recursive descent parsing
                let tree = parser.analyze();

                println!("\nMTree:");
                tree.print();
            },
            "execute" => {
                println!("Execute given file.");

                if args.is_empty() { println!("File must be given") }
                let file_path = &args[0];
                println!("Running parser to tokenize & parse a file {}:", file_path);

                // create recursive descent parser
                let mut lexer = Lexer::new();
                lexer.set_input(String::from(file_path.clone()));
                let mut parser = Parser::new(lexer);

                // start recursive descent parsing
                let tree = parser.analyze();

                println!("\nMTree:");
                tree.print();

                // --------------------------------------------------------
                // analyze tree
                // --------------------------------------------------------
                println!("----------------------------------------------------------------");
                let analyzer = Analyzer::new();
                let rc_tree_analyzed = analyzer.analyze_global(Rc::new(tree.clone()));
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
            "example" => {
                if args.is_empty() { println!("Example choice not given.") }
                let example_choice = args[0];

                match example_choice {
                    "OHL" => {
                        ohl_analyzer_evaluator_sample_function()
                    }
                    "YARRICK" => {
                        yarrick_analyzer_evaluator_sample_function()
                    }
                    _ => {
                        println!("Invalid example choice given.")
                    }
                }
            }
            _ => println!("Unknown command: {}", cmd)
        }
    }

    return;
}

fn ohl_analyzer_evaluator_sample_function() {
    // --------------------------------------------------------
    // Example Program
    // --------------------------------------------------------
    //
    // func fac(n)
    // [
    //     write n;
    //     if n <= 2 [
    //         return 1;
    //     ] else [
    //         return n * fac(n-1);
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
            println!("parse <file>: \n- tokenizes the input from the file and then parses the tokens.\n- Prints out the tree form of the function.\n");
        }
        "execute" => {
            println!("execute <file>: \n- The whole shebang.\n- tokenizes, parses, analyzes, and then executes the given file.\n- Prints output for each step.");
        }
        "example" => {
            println!("example <\"OHL\" | \"YARRICK\">: \n-prints one of two examples that utilize the analyzer and executor on a predefined tree.");
        }
        _ => {
            println!("Unknown command: {}.\n", command);
        }
    }
}

fn list_command(command: &str) {
    let print_all_string = String::from("All commands:\n\t help \n\t print \n\t list [commands] \n\t list tokens \n\t example \n\t tokenize \n\t parse \n\t execute \n\t example");
    match command {
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

fn RunLexerOnFile(lex: &mut Lexer) {
    lex.advance();
    lex.print_token();

    while lex.token.clone().unwrap() != TCode::EOI {
        lex.advance();
        lex.print_token();
    }
}

fn print(file_name: String, flag : String) {
    if flag == "--numbered" {
        let mut count : i16 = 1;

        for line in read_to_string(file_name).unwrap().lines() {
            println!("{} {}", count, line.to_string());
            count += 1;
        }
    }
    else {
        for line in read_to_string(file_name).unwrap().lines() {
            println!("{}", line.to_string());
        }
    }
}