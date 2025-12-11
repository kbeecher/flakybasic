use std::collections::HashMap;

use crate::{
    errors::BasicError,
    statement::{ProgramSignal, Statement, find_line},
};

/// Run a program from the beginning.
///
/// # Arguments
/// * `lines` - A mapping of line numbers to statements
/// * `variables` - The variables table
///
pub fn run(
    variables: &mut HashMap<char, i32>,
    program: &Vec<(i32, Statement)>,
) -> Option<BasicError> {
    let mut pc = 0;
    let mut running = true;
    let mut stack: Vec<usize> = Vec::new();

    let program_size = program.len();

    // Execution will continue until the PC reaches the last line or something
    // alters running status (e.g. the 'end' command).
    while pc < program_size && running == true {
        let s = program.get(pc).unwrap();

        match execute_line(&s.1, &mut pc, &mut running, variables, &mut stack, &program) {
            Some(e) => {
                return Some(e);
            }
            None => (),
        }
    }

    None
}

pub fn execute_line(
    statement: &Statement,
    pc: &mut usize,
    running: &mut bool,
    variables: &mut HashMap<char, i32>,
    stack: &mut Vec<usize>,
    program: &Vec<(i32, Statement)>,
) -> Option<BasicError> {
    match statement.execute(variables) {
        Ok(maybe_flow) => match maybe_flow {
            None => {
                *pc += 1;
            }

            Some(f) => match f {
                ProgramSignal::Jump(line_num) => match find_line(&program, line_num) {
                    Some(new_line) => {
                        *pc = new_line;
                    }
                    None => {
                        let bad_line = program.get(*pc).expect("Unrecoverable error");
                        return Some(BasicError::RuntimeError(format!(
                            "Unknown line number in line {}",
                            bad_line.0
                        )));
                    }
                },

                ProgramSignal::Call(line_num) => {
                    stack.push(*pc);
                    match find_line(&program, line_num) {
                        Some(new_line) => *pc = new_line,
                        None => {
                            let bad_line = program.get(*pc).expect("Unrecoverable error");
                            return Some(BasicError::RuntimeError(format!(
                                "Unknown line number in line {}",
                                bad_line.0
                            )));
                        }
                    }
                }

                ProgramSignal::Return => match stack.pop() {
                    Some(address) => {
                        *pc = address;
                        *pc += 1;
                    }
                    None => {
                        let bad_line = program.get(*pc).expect("Unrecoverable error");
                        return Some(BasicError::RuntimeError(format!(
                            "Return without gosub in line {}",
                            bad_line.0
                        )));
                    }
                },

                ProgramSignal::List => {
                    for line in program.iter() {
                        println!("{} {}", line.0, line.1);
                    }
                }

                ProgramSignal::Run => {
                    run(variables, program);
                }

                ProgramSignal::End => {
                    *running = false;
                }
            },
        },
        Err(e) => match e {
            // Wrap errors to give line number info to user.
            BasicError::SyntaxError(e) => {
                let line = program.get(*pc).expect("Error").0;
                return Some(BasicError::SyntaxError(format!(
                    "Syntax error in {}: {}",
                    line, e
                )));
            }
            BasicError::RuntimeError(e) => {
                let line = program.get(*pc).expect("Error").0;
                return Some(BasicError::RuntimeError(format!(
                    "Runtime error in {}: {}",
                    line, e
                )));
            }
        },
    }

    None
}
