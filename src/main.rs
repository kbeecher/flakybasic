use clap::Parser;
use std::{
    collections::HashMap,
    io::{self},
    path::PathBuf,
};

use crate::{
    errors::BasicError, executor::execute_line, parser::parse_line, program::update_program,
    statement::Statement,
};

mod errors;
mod executor;
mod expression;
mod parser;
mod program;
mod statement;

#[derive(Parser, Debug)]
struct Args {
    filename: Option<PathBuf>,
}

fn main() {
    let mut pc = 0;
    let mut running: bool;
    let mut stack: Vec<usize> = Vec::new();
    let mut variables: HashMap<char, i32> = HashMap::new();
    let mut program: Vec<(i32, Statement)> = Vec::new();

    let mut input_line = String::new();
    println!("ready.");

    loop {
        input_line.clear();

        io::stdin()
            .read_line(&mut input_line)
            .expect("Failed to read line");

        match parse_line(input_line.clone()) {
            Ok(prog_line) => match prog_line.0 {
                None => {
                    running = true;
                    match execute_line(
                        &prog_line.1,
                        &mut pc,
                        &mut running,
                        &mut variables,
                        &mut stack,
                        &program,
                    ) {
                        None => (),
                        Some(e) => match e {
                            BasicError::SyntaxError(e) => println!("{}", e),
                            BasicError::RuntimeError(e) => println!("{}", e),
                        },
                    }
                }
                Some(line_num) => {
                    update_program(&mut program, (line_num, prog_line.1));
                }
            },
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
}
