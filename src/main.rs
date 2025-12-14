use clap::Parser;
use std::{
    collections::HashMap,
    io::{self},
    path::PathBuf,
};

use crate::{
    errors::BasicError, executor::execute_immediate, parser::SourceReader, program::update_program,
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

fn report_error(err: BasicError, line_num: Option<i32>) {
    match err {
        BasicError::SyntaxError(s) => {
            print!("Syntax error: {}", s);
            if let Some(n) = line_num {
                print!(" in line {}", n);
            }
            println!()
        }
        BasicError::RuntimeError(s) => {
            print!("Runtime error: {}", s);
            if let Some(n) = line_num {
                print!(" in line {}", n);
            }
            println!()
        }
    }
}

fn main() {
    let mut variables: HashMap<char, i32> = HashMap::new();
    let mut program: Vec<(i32, Statement)> = Vec::new();

    let mut input_line = String::new();
    println!("Ready.");

    loop {
        input_line.clear();

        io::stdin()
            .read_line(&mut input_line)
            .expect("Failed to read line");

        //
        // Parse line
        //

        let mut reader = SourceReader::new(input_line.clone());
        reader.skip_ws();

        // Get line number
        let line_num: Option<i32> = match reader.is_digit() {
            false => None,
            true => {
                let res = reader.get_number();
                match res {
                    Ok(n) => Some(n),
                    Err(e) => {
                        report_error(e, None);
                        continue;
                    }
                }
            }
        };

        // Build the statement
        let statement: (Option<i32>, Statement) = match reader.build_statement() {
            Ok(s) => match line_num {
                Some(n) => (Some(n), s),
                None => (None, s),
            },
            Err(e) => match line_num {
                Some(n) => {
                    report_error(e, Some(n));
                    continue;
                }
                None => {
                    report_error(e, None);
                    continue;
                }
            },
        };

        //
        // Run the line or update the program?
        //

        if statement.0.is_some() {
            // There's a line number, so update the program.
            update_program(
                &mut program,
                (
                    statement
                        .0
                        .expect("Unrecoverable error getting line number"),
                    statement.1,
                ),
            );
        } else {
            // There's no line number, so execute it in immediate mode.
            match execute_immediate(&statement.1, &mut variables, &mut program) {
                None => (),
                Some(err) => {
                    report_error(err, None);
                }
            }
        }
    }
}
