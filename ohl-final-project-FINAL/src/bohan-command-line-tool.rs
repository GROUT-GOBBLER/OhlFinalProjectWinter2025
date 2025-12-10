#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(warnings)]
#![allow(non_snake_case)]

mod hw_assignment_2;
mod hw_assignment_3;
mod hw_assignment_4;

use hw_assignment_2::*;
use hw_assignment_3::*;
use hw_assignment_4::*;


use std::env;
use std::fs;
use rand::{random, Rng};
use std::io::{BufRead, BufReader};
use std::error::Error;


fn print_help_for(command: &str) {
    match command {
        "help" => {
            println!("help [command]: \n- prints help info for commands\n");
        }
        "print" => {
            println!("print <file> [--numbered]: \n- prints out the contents of a file to the output. If numbered is true, then it will list every line with a number before it\n");
        }
        "list" => {
            println!("list commands OR list rules OR list tokens: \n- prints the list of all commands with or without the second field being entered as [commands]\n- with the 2nd part being [rules], it prints the grammar rules\n- if the 2nd field is [tokens], will print out all tokens in the token enum\n");
        }
        "derive" => {
            println!("derive random [limit of derivation steps]: \n- generates and prints a random word, you can enter a derivation step limit, otherwise the default limit is grammar.rules.len() as i32 \n");
            println!("derive <int-list> [sequence of numbers]: \n- manually generates a word by using a sequence of numbers inputted by the user that it will use as indexes to select rules\n");
        }
        "example" => {
            println!("example: \n- runs an example demonstrating the grammar\n");
        }
        "tokenize" => {
            println!("tokenize <file>: \n- tokenizes the input from a file and then prints out the token form of the function\n");
        }
        "parse" => {
            println!("parse <file>: \n- tokenizes & parses the input from a file and then prints out resulting parse tree\n");
        }
        _ => {
            println!("Unknown command: {}.\n", command);
        }
    }
}


fn main() {

    let grammar = create_operator_grammar();

    let args: Vec<String> = env::args().skip(1).collect();
    let valid_commands: Vec<String> = vec![
        String::from("help"),
        String::from("print"),
        String::from("list"),
        String::from("derive"),
        String::from("example"),
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
            let print_all_string = String::from("All commands:\n\t help \n\t print \n\t list [commands] \n\t list rules \n\t list tokens \n\t derive (random) \n\t derive (int-list) \n\t example \n\t tokenize \n\t parse");
            if args.len() < 2 {
                println!("{}", print_all_string);
            }
            else {
                let cmd = &args[1];
                match cmd.as_str() {
                    "commands" => {
                        println!("{}", print_all_string);
                    }
                    "rules" => {
                        println!("Grammar rules:");
                        for (index, rule) in grammar.rules.iter().enumerate() {
                            println!("{}: {} -> {}", index, rule.lhs, rule.rhs);
                        }
                    }
                    "tokens" => {
                        println!("====Tokens====");
                        println!("Brackets: PARENS_L, PARENS_R, BRACKET_L, BRACKET_R, BRACE_L, BRACE_R\n\
                                  Separators: POINT, COMMA, COLON, SEMICOLON, ARROW_R\n\
                                  Arithmetic Ops: ADD, SUB, MUL, DIV\n\
                                  Relational Ops: EQ, LT, GT, NEQ, NLT, NGT\n\
                                  Logical Ops: NOT, AND, OR\n\
                                  Assignment: ASSIGN\n\
                                  Keywords: FUNC, LET, IF, ELSE, ELSE_IF, WHILE, PRINT, RETURN\n\
                                  Identifiers: ID\n\
                                  Basic Types: TYPE_INT32, TYPE_FLT32, TYPE_CHAR\n\
                                  Literals: LIT_INT32, LIT_FLT32, LIT_CHAR, LIT_STRING, TRUE, FALSE\n\
                                  Other Stuff: AMPERSAND, MUT, LINEBREAK, ERROR\n\
                                  End-of-Input: EOI\n\
                                  Meta Operators: META_PROGRAM, META_FUNC, META_PARAM_LIST, META_PARAM, META_LET, META_RETURN, META_IF, META_ELSE_IF, META_BLOCK, META_VOID, META_INFER\n");
                    }
                    _ => {
                        println!("{}", print_all_string);
                    }
                }
            }
        }
        "derive" => {
            if args.len() < 2 {
                println!("Error: 'derive' requires random or rule indices. \n Usage: derive random [optional limit], or derive <int-list> (a sequence of integers separated by a space corresponding to rule indexes of the grammar, you can try the command: list rules, to see what the grammar is");
                return;
            }
            if args[1] == "random" {
                let maximum_step_limit: i32 = grammar.rules.len() as i32;
                let mut step_limit: i32 = random();
                if args.len() > 2 {
                    match args[2].parse::<i32>() {
                        Ok(limit) => {
                            if limit > maximum_step_limit {
                                step_limit = maximum_step_limit;
                            }
                            else {
                                step_limit = limit;
                            }
                        }
                        Err(_) => {
                            println!("Invalid step limit: {}", args[2]);
                            return;
                        }
                    }
                }
                else {
                    step_limit = rand::thread_rng().gen_range(1..=grammar.rules.len() as i32);
                }
                Derivation::print_random(&grammar, Option::from(step_limit as usize));
            }
            // User inputs stuff
            else {
                let mut rule_idxs: Vec<usize> = Vec::new();
                for arg in &args[1..] {
                    match arg.parse::<usize>() {
                        Ok(idx) => rule_idxs.push(idx),
                        Err(_) => {
                            println!("Invalid rule index: {}", arg);
                            return;
                        }
                    }
                }
                let mut deriv = Derivation::new(&grammar);
                for &idx in &rule_idxs {
                    if let Err(e) = deriv.derive_leftmost(&grammar, idx as i32) {
                        println!("Derivation error: {:?}", e);
                        return;
                    }
                }
                if deriv.is_complete() {
                    if let Ok(word) = deriv.word() {
                        println!("Derived word: {}", word);
                    }
                }
                else {
                    let current_form = &deriv.steps.last().unwrap().1.form;
                    println!("Incomplete derivation: current form {}", current_form);
                }
            }
        }
        "example" => {
            println!("Running manual derivation example template:");
            example_manual();
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
        _ => {
            println!("Unknown command: {}.", args[0]);
        }
    }
}