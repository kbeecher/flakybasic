use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, Write},
};

use crate::{
    errors::BasicError,
    parser::SourceReader,
    program::update_program,
    statement::{ProgramSignal, Statement, find_line},
};

/// Run a program from the beginning.
///
/// # Arguments
/// * `program` - The program to run
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

        match execute_indirect(&s.1, &mut pc, &mut running, variables, &mut stack, &program) {
            Some(e) => {
                return Some(e);
            }
            None => (),
        }
    }

    None
}

/// Execute a single statement immediately.
///
/// # Arguments
/// * `statement` - The statement to execute
/// * `variables` - The variable table
/// * `program` - The current state of the program
pub fn execute_immediate(
    statement: &Statement,
    variables: &mut HashMap<char, i32>,
    program: &mut Vec<(i32, Statement)>,
) -> Option<BasicError> {
    match statement.execute(variables) {
        Err(e) => Some(e),

        Ok(signal) => match signal {
            None => None,

            Some(ProgramSignal::List) => {
                for line in program.iter() {
                    println!("{} {}", line.0, line.1);
                }

                return None;
            }

            Some(ProgramSignal::Run) => {
                return run(variables, program);
            }

            Some(ProgramSignal::Load(filename)) => {
                let src_file = match File::open(filename) {
                    Ok(file) => file,
                    Err(err) => return Some(BasicError::RuntimeError(format!("{}", err))),
                };

                let reader = BufReader::new(src_file);

                for line in reader.lines() {
                    match line {
                        Err(err) => {
                            return Some(BasicError::RuntimeError(format!(
                                "File read error: {}",
                                err
                            )));
                        }
                        Ok(src_line) => {
                            let mut reader = SourceReader::new(src_line.clone());
                            reader.skip_ws();

                            // Get line number
                            if !reader.is_digit() {
                                return Some(BasicError::RuntimeError(String::from(
                                    "Line number missing in file",
                                )));
                            }

                            let line_num = match reader.get_number() {
                                Err(e) => {
                                    return Some(e);
                                }
                                Ok(n) => n,
                            };

                            // Build the line
                            let line: (i32, Statement) = match reader.build_statement() {
                                Ok(s) => (line_num, s),
                                Err(e) => {
                                    return Some(e);
                                }
                            };

                            // Update the program
                            update_program(program, line);
                        }
                    }
                }

                println!("File loaded.");

                return None;
            }

            Some(ProgramSignal::Save(filename)) => {
                let mut file = match File::create(filename) {
                    Ok(f) => f,
                    Err(e) => {
                        return Some(BasicError::RuntimeError(format!("File read error: {}", e)));
                    }
                };

                for line in program.iter() {
                    match writeln!(file, "{} {}", line.0, line.1) {
                        Ok(_) => (),
                        Err(e) => {
                            return Some(BasicError::RuntimeError(format!(
                                "File read error: {}",
                                e
                            )));
                        }
                    }
                }

                println!("File saved.");

                return None;
            }

            Some(ProgramSignal::Jump(_))
            | Some(ProgramSignal::Call(_))
            | Some(ProgramSignal::Return)
            | Some(ProgramSignal::End) => {
                return Some(BasicError::RuntimeError(String::from(
                    "Cannot execute this command outside of a program.",
                )));
            }
        },
    }
}

/// Execute a line as part of a running program.
///
/// # Arguments
/// * `statement` - The statement to execute
/// * `pc` - Current program counter value
/// * `running` - Flag indicating whether program is currently running
/// * `variables` - The variable table
/// * `stack` -  The call stack
/// * `program` - The program being executed
pub fn execute_indirect(
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
                    return Some(BasicError::RuntimeError(String::from(
                        "Cannot list a program during execution.",
                    )));
                }

                ProgramSignal::Run => {
                    return Some(BasicError::RuntimeError(String::from(
                        "Cannot run a program that's already in execution.",
                    )));
                }

                ProgramSignal::Load(_) => {
                    return Some(BasicError::RuntimeError(String::from(
                        "Cannot load a program during execution.",
                    )));
                }

                ProgramSignal::Save(_) => {
                    return Some(BasicError::RuntimeError(String::from(
                        "Cannot save a program during execution.",
                    )));
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
