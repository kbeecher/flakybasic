use core::fmt;
use std::{
    collections::HashMap,
    fmt::Display,
    io::{self, Write},
};

use crate::{
    errors::BasicError,
    expression::{Condition, Expression, Number, Relop, eval_expression},
    parser::{
        END, FOR, GOSUB, GOTO, IF, INPUT, LET, LIST, NEXT, PRINT, REM, RETURN, RUN, STEP, TO,
    },
};

/// Find the index of the line with the given line number. The index can
/// be used as a value for the Executor's program counter.
pub fn find_line(program: &Vec<(i32, Statement)>, line_num: i32) -> Option<usize> {
    for (pc, line) in program.iter().enumerate() {
        if line.0 == line_num {
            return Some(pc);
        }
    }

    return None;
}

/// Actions that signal back to the executor that it should take action,
/// either to alter flow of program control (e.g. via jumps, gosubs,
/// returns, program termination etc.) or carry out some immediate
/// command.
#[derive(Debug, PartialEq)]
pub enum ProgramSignal {
    Jump(i32),
    Call(i32),
    Return,
    // Signal = var_name, start_val, end_val, step_val
    StartLoop(char, i32, i32, Option<i32>),
    EndLoop,
    List,
    Load(String),
    Save(String),
    Run,
    End,
}

/// A statement in a program.
pub enum Statement {
    Empty,
    Rem(String),
    Print(Vec<Expression>),
    Let(char, Expression),
    If(Condition, Box<Statement>),
    Goto(i32),
    Input(char),
    Gosub(i32),
    Return,
    For(char, Expression, Expression, Option<Expression>),
    Next,
    List,
    Load(String),
    Save(String),
    Run,
    End,
}

impl Statement {
    /// Execute the statement
    ///
    /// # Arguments
    /// * `variables` - the variables table
    ///
    /// # Returns
    /// * Either an optional program flow or an error
    pub fn execute(
        &self,
        variables: &mut HashMap<char, Number>,
    ) -> Result<Option<ProgramSignal>, BasicError> {
        match self {
            Self::Empty => return Ok(None),

            Self::Rem(_) => {
                return Ok(None);
            }

            // Print the supplied expressions (if any).
            Self::Print(args) => {
                for arg in args.iter() {
                    match arg {
                        Expression::String(s) => {
                            print!("{}", s);
                        }
                        Expression::Numeric(n) => {
                            print!("{}", n);
                        }
                        Expression::Variable(c) => {
                            print!("{}", eval_expression(Expression::Variable(*c), variables)?);
                        }
                        Expression::Operator(op, l_exp, r_exp) => {
                            print!(
                                "{}",
                                eval_expression(
                                    Expression::Operator(*op, l_exp.clone(), r_exp.clone()),
                                    variables
                                )?
                            )
                        }
                    }
                }

                println!();
            }

            // Variable assignment command.
            Self::Let(var, value) => match value {
                Expression::String(_) => {
                    return Err(BasicError::RuntimeError(String::from(
                        "Can't assign strings to variables",
                    )));
                }
                Expression::Numeric(n) => {
                    variables.insert(*var, *n);
                }
                Expression::Variable(c) => {
                    variables.insert(*var, eval_expression(Expression::Variable(*c), variables)?);
                }
                Expression::Operator(op, l_exp, r_exp) => {
                    variables.insert(
                        *var,
                        eval_expression(
                            Expression::Operator(*op, l_exp.clone(), r_exp.clone()),
                            variables,
                        )?,
                    );
                }
            },

            // If statement takes a condition and a consequent statement
            // that's executed when the condition evaluates to true.
            Self::If(condition, consequent) => match condition {
                Condition::Boolean(l_exp, relop, r_exp) => {
                    // Evaluate the left-hand and right-hand expressions.
                    let l_val: Number = match l_exp {
                        Expression::String(_) => {
                            return Err(BasicError::RuntimeError(String::from(
                                "Can't compare strings",
                            )));
                        }
                        Expression::Numeric(n) => {
                            eval_expression(Expression::Numeric(*n), variables)?
                        }

                        Expression::Variable(v) => {
                            eval_expression(Expression::Variable(*v), variables)?
                        }

                        Expression::Operator(op, l_exp, r_exp) => eval_expression(
                            Expression::Operator(*op, l_exp.clone(), r_exp.clone()),
                            variables,
                        )?,
                    };
                    let r_val: Number = match r_exp {
                        Expression::String(_) => {
                            return Err(BasicError::RuntimeError(String::from(
                                "Can't compare strings",
                            )));
                        }
                        Expression::Numeric(n) => {
                            eval_expression(Expression::Numeric(*n), variables)?
                        }
                        Expression::Variable(v) => {
                            eval_expression(Expression::Variable(*v), variables)?
                        }
                        Expression::Operator(op, l_exp, r_exp) => eval_expression(
                            Expression::Operator(*op, l_exp.clone(), r_exp.clone()),
                            variables,
                        )?,
                    };

                    // Once the two expressions are evaluated, test the condition.
                    if self.eval_relop(l_val, *relop, r_val) {
                        // If true, then execute the consequent.
                        return consequent.execute(variables);
                    }
                }
            },

            // Unconditional jump.
            Self::Goto(n) => return Ok(Some(ProgramSignal::Jump(*n))),

            // Take input from the user. Current supports integer only.
            Self::Input(v) => {
                // Print a prompt.
                print!("? ");
                io::stdout().flush().unwrap();

                let mut buffer = String::new();
                let stdin = io::stdin();
                let res = stdin.read_line(&mut buffer);

                match res {
                    // Parse the input into a number
                    Ok(_) => match buffer.trim().parse::<i32>() {
                        Ok(i) => {
                            variables.insert(*v, Number::Integer(i));
                        }
                        Err(_) => match buffer.trim().parse::<f64>() {
                            Ok(f) => {
                                variables.insert(*v, Number::Float(f));
                            }
                            Err(_) => {
                                return Err(BasicError::RuntimeError(String::from("Parse error")));
                            }
                        },
                    },
                    Err(_) => {
                        return Err(BasicError::RuntimeError(String::from("Input error")));
                    }
                }
            }

            // Goto subroutine
            Self::Gosub(line_num) => return Ok(Some(ProgramSignal::Call(*line_num))),

            // Return from subroutine
            Self::Return => return Ok(Some(ProgramSignal::Return)),

            // Begin next iteration of a loop
            Self::For(var, start_val, end_val, maybe_step_val) => {
                let step_val: Option<i32> = match maybe_step_val {
                    None => None,
                    Some(exp) => {
                        let final_step_val = eval_expression(exp.clone(), variables)?;
                        if !final_step_val.is_int() {
                            return Err(BasicError::RuntimeError(String::from(
                                "Values in for statement must be integers",
                            )));
                        } else {
                            Some(final_step_val.int_value().expect("Type error"))
                        }
                    }
                };

                let final_start_val = eval_expression(start_val.clone(), variables)?;
                let final_end_val = eval_expression(end_val.clone(), variables)?;

                if !final_start_val.is_int() || !final_end_val.is_int() {
                    return Err(BasicError::RuntimeError(String::from(
                        "Values in for statement must be integers",
                    )));
                } else {
                    return Ok(Some(ProgramSignal::StartLoop(
                        *var,
                        final_start_val.int_value().expect("Type error"),
                        final_end_val.int_value().expect("Type error"),
                        step_val,
                    )));
                }
            }

            // Evaluate whether to continue with another loop
            Self::Next => return Ok(Some(ProgramSignal::EndLoop)),

            // List the program
            Self::List => return Ok(Some(ProgramSignal::List)),

            // Load a program
            Self::Load(filename) => return Ok(Some(ProgramSignal::Load(filename.clone()))),

            // Save a program
            Self::Save(filename) => return Ok(Some(ProgramSignal::Save(filename.clone()))),

            // Run the program
            Self::Run => return Ok(Some(ProgramSignal::Run)),

            Self::End => return Ok(Some(ProgramSignal::End)),
        }

        return Ok(None);
    }

    fn eval_relop(&self, l_val: Number, relop: Relop, r_val: Number) -> bool {
        match relop {
            Relop::EQ => l_val == r_val,
            Relop::NEQ => l_val != r_val,
            Relop::LT => l_val < r_val,
            Relop::LTE => l_val <= r_val,
            Relop::GT => l_val > r_val,
            Relop::GTE => l_val >= r_val,
        }
    }
}

impl Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Statement::Rem(c) => write!(f, "{} {}", REM, c),
            Statement::Print(args) => {
                let mut output = String::from(format!("{} ", PRINT));
                let mut first = true;

                for arg in args.iter() {
                    if !first {
                        output.push_str(", ");
                    }
                    output.push_str(&format!("{}", arg));
                    first = false;
                }

                write!(f, "{}", output)
            }
            Statement::Let(var, exp) => write!(f, "{} {}={}", LET, var, exp),
            Statement::If(con, stmnt) => write!(f, "{} {} {}", IF, con, stmnt),
            Statement::Goto(num) => write!(f, "{} {}", GOTO, num),
            Statement::Input(var) => write!(f, "{} {}", INPUT, var),
            Statement::Gosub(num) => write!(f, "{} {}", GOSUB, num),
            Statement::Return => write!(f, "{}", RETURN),
            Statement::For(var, start_val, end_val, step_val) => {
                if let Some(v) = step_val {
                    write!(
                        f,
                        "{} {}={} {} {} {} {}",
                        FOR, var, start_val, TO, end_val, STEP, v
                    )
                } else {
                    write!(f, "{} {}={} {} {}", FOR, var, start_val, TO, end_val)
                }
            }
            Statement::Next => write!(f, "{}", NEXT),
            Statement::List => write!(f, "{}", LIST),
            Statement::Run => write!(f, "{}", RUN),
            Statement::Load(_) => Ok(()),
            Statement::Save(_) => Ok(()),
            Statement::End => write!(f, "{}", END),
            Statement::Empty => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expression::ArithOp;

    #[test]
    fn prints_string() {
        let mut variables: HashMap<char, Number> = HashMap::new();
        let mut lines: HashMap<i32, Statement> = HashMap::new();

        lines.insert(
            10,
            Statement::Print(vec![Expression::String(String::from("Hello, world!"))]),
        );

        match lines.get(&10).expect("Error").execute(&mut variables) {
            Ok(maybe_flow) => assert!(maybe_flow.is_none()),
            Err(e) => panic!("{}", e),
        }
    }

    #[test]
    fn assigns_variable() {
        let mut variables: HashMap<char, Number> = HashMap::new();
        let mut lines: HashMap<i32, Statement> = HashMap::new();

        lines.insert(
            10,
            Statement::Let('A', Expression::Numeric(Number::Integer(42))),
        );

        match lines.get(&10).expect("Error").execute(&mut variables) {
            Ok(maybe_flow) => assert!(maybe_flow.is_none()),
            Err(e) => panic!("{}", e),
        }

        assert_eq!(*variables.get(&'A').unwrap(), Number::Integer(42));
    }

    #[test]
    fn evaluates_condition() {
        let mut variables: HashMap<char, Number> = HashMap::new();
        let mut lines: HashMap<i32, Statement> = HashMap::new();

        variables.insert('A', Number::Integer(42));

        lines.insert(
            20,
            Statement::If(
                Condition::Boolean(
                    Expression::Variable('A'),
                    Relop::EQ,
                    Expression::Numeric(Number::Integer(42)),
                ),
                Box::new(Statement::Let(
                    'A',
                    Expression::Numeric(Number::Integer(69)),
                )),
            ),
        );

        match lines.get(&20).expect("Error").execute(&mut variables) {
            Ok(maybe_flow) => assert!(maybe_flow.is_none()),
            Err(e) => panic!("{}", e),
        }

        assert_eq!(*variables.get(&'A').unwrap(), Number::Integer(69));
    }

    #[test]
    fn evaluates_complex_expressions() {
        let mut variables: HashMap<char, Number> = HashMap::new();
        let mut lines: HashMap<i32, Statement> = HashMap::new();

        let exp = Expression::Operator(
            ArithOp::Add,
            Some(Box::new(Expression::Numeric(Number::Integer(2)))),
            Some(Box::new(Expression::Operator(
                ArithOp::Multiply,
                Some(Box::new(Expression::Numeric(Number::Integer(3)))),
                Some(Box::new(Expression::Numeric(Number::Integer(4)))),
            ))),
        );

        lines.insert(10, Statement::Let('N', exp));

        match lines.get(&10).expect("Error").execute(&mut variables) {
            Ok(maybe_flow) => assert!(maybe_flow.is_none()),
            Err(e) => panic!("{}", e),
        }

        assert_eq!(*variables.get(&'N').unwrap(), Number::Integer(14));
    }

    #[test]
    fn branches_unconditionally() {
        let mut variables: HashMap<char, Number> = HashMap::new();
        let mut lines: HashMap<i32, Statement> = HashMap::new();

        variables.insert('X', Number::Integer(1));

        lines.insert(10, Statement::Goto(30));
        lines.insert(
            20,
            Statement::Let('X', Expression::Numeric(Number::Integer(2))),
        );
        lines.insert(
            30,
            Statement::Let('X', Expression::Numeric(Number::Integer(3))),
        );

        match lines.get(&10).expect("Error").execute(&mut variables) {
            Ok(maybe_flow) => match maybe_flow.unwrap() {
                ProgramSignal::Jump(f) => assert_eq!(f, 30),
                _ => panic!("Wrong type of program flow"),
            },
            Err(e) => panic!("{}", e),
        }
    }
}
