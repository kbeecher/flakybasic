use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, Write},
};

use crate::{
    errors::BasicError,
    expression::Number,
    parser::SourceReader,
    program::{find_line, update_program},
    statement::{ProgramSignal, Statement},
};

/// Run a program from the beginning.
///
/// # Arguments
/// * `program` - The program to run
/// * `variables` - The variables table
///
pub fn run(
    variables: &mut HashMap<char, Number>,
    program: &Vec<(i32, Statement)>,
) -> Option<BasicError> {
    let mut pc = 0;
    let mut running = true;
    let mut stack: Vec<usize> = Vec::new();
    let mut loop_stack: Vec<(char, i32, i32, usize)> = Vec::new();

    let program_size = program.len();

    // Execution will continue until the PC reaches the last line or something
    // alters running status (e.g. the 'end' command).
    while pc < program_size && running == true {
        let s = program.get(pc).unwrap();
        match execute_indirect(
            &s.1,
            &mut pc,
            &mut running,
            variables,
            &mut stack,
            &mut loop_stack,
            &program,
        ) {
            None => (),
            Some(e) => return Some(e),
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
    variables: &mut HashMap<char, Number>,
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

                            let line_num = match reader.get_integer() {
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
            | Some(ProgramSignal::StartLoop(_, _, _, _))
            | Some(ProgramSignal::EndLoop)
            | Some(ProgramSignal::End) => {
                return Some(BasicError::RuntimeError(String::from(
                    "Cannot execute this command outside of a program",
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
    variables: &mut HashMap<char, Number>,
    stack: &mut Vec<usize>,
    loop_stack: &mut Vec<(char, i32, i32, usize)>,
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

                ProgramSignal::StartLoop(var, start_val, end_val, maybe_step_val) => {
                    // Are we entering the loop (i.e. is the var at the top of the
                    // loop stack different from that in the current line? If so,
                    // set up the stack entry.

                    // If there's no step value. we default to 1. Exception: if
                    // end_val < start_val, make step = -1.
                    let step_val = match maybe_step_val {
                        Some(v) => v,
                        None => {
                            if end_val < start_val {
                                -1
                            } else {
                                1
                            }
                        }
                    };

                    // Step value can't be zero.
                    if step_val == 0 {
                        return Some(BasicError::RuntimeError(String::from(
                            "Step value cannot be zero",
                        )));
                    }

                    // Stack entry: (char, i32, i32, usize) =
                    // variable, end_val, step_val, PC at for statement
                    if loop_stack.is_empty()
                        || loop_stack.last().expect("Error executing for").0 != var
                    {
                        variables.insert(var, Number::Integer(start_val));
                        loop_stack.push((var, end_val, step_val, *pc));
                    }

                    *pc += 1;
                }

                ProgramSignal::EndLoop => {
                    // Increment the loop variable.
                    match loop_stack.last() {
                        None => {
                            return Some(BasicError::RuntimeError(String::from(
                                "Next without for",
                            )));
                        }
                        Some(entry) => {
                            variables.insert(
                                entry.0,
                                variables[&entry.0.clone()] + Number::Integer(entry.2),
                            );

                            // Has it reached the end val? Positive stepping
                            // means we must be above the end val; negative
                            // stepping means we must be below the end val.
                            let end_reached = match entry.2.is_negative() {
                                true => variables[&entry.0] < Number::Integer(entry.1),
                                false => variables[&entry.0] > Number::Integer(entry.1),
                            };

                            // If it's reached the end val, then pop the loop stack and
                            // proceed to next line.
                            if end_reached {
                                loop_stack.pop();
                                *pc += 1;
                            } else {
                                // Otherwise, jump to the top of the loop.
                                *pc = entry.3;
                            }
                        }
                    }
                }

                ProgramSignal::List => {
                    return Some(BasicError::RuntimeError(String::from(
                        "Cannot list a program during execution",
                    )));
                }

                ProgramSignal::Run => {
                    return Some(BasicError::RuntimeError(String::from(
                        "Cannot run a program that's already in execution",
                    )));
                }

                ProgramSignal::Load(_) => {
                    return Some(BasicError::RuntimeError(String::from(
                        "Cannot load a program during execution",
                    )));
                }

                ProgramSignal::Save(_) => {
                    return Some(BasicError::RuntimeError(String::from(
                        "Cannot save a program during execution",
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
                return Some(BasicError::SyntaxError(format!("{} in line {}", e, line)));
            }
            BasicError::RuntimeError(e) => {
                let line = program.get(*pc).expect("Error").0;
                return Some(BasicError::RuntimeError(format!("{} in line {}", e, line)));
            }
        },
    }

    None
}
