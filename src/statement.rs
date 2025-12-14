use core::fmt;
use std::{
    collections::HashMap,
    fmt::Display,
    io::{self, Write},
};

use crate::{
    errors::BasicError,
    expression::{Condition, Expression, Relop, eval_expression},
    parser::{END, GOSUB, GOTO, IF, INPUT, LET, LIST, PRINT, REM, RETURN, RUN},
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
        variables: &mut HashMap<char, i32>,
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
                            print!("{}", s)
                        }
                        Expression::Integer(i) => {
                            print!("{}", eval_expression(Expression::Integer(*i), variables)?);
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
                Expression::Integer(i) => {
                    variables.insert(*var, eval_expression(Expression::Integer(*i), variables)?);
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
                    let l_val: i32 = match l_exp {
                        Expression::String(_) => {
                            return Err(BasicError::RuntimeError(String::from(
                                "Can't compare strings",
                            )));
                        }
                        Expression::Integer(i) => {
                            eval_expression(Expression::Integer(*i), variables)?
                        }
                        Expression::Variable(v) => {
                            eval_expression(Expression::Variable(*v), variables)?
                        }
                        Expression::Operator(op, l_exp, r_exp) => eval_expression(
                            Expression::Operator(*op, l_exp.clone(), r_exp.clone()),
                            variables,
                        )?,
                    };
                    let r_val: i32 = match r_exp {
                        Expression::String(_) => {
                            return Err(BasicError::RuntimeError(String::from(
                                "Can't compare strings",
                            )));
                        }
                        Expression::Integer(i) => {
                            eval_expression(Expression::Integer(*i), variables)?
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
                    Ok(_) => match buffer.trim().parse() {
                        Ok(n) => {
                            variables.insert(*v, n);
                        }
                        Err(_) => {
                            return Err(BasicError::RuntimeError(String::from("Parse error")));
                        }
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

    fn eval_relop(&self, l_val: i32, relop: Relop, r_val: i32) -> bool {
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
            Statement::List => write!(f, "{}", LIST),
            Statement::Run => write!(f, "{}", RUN),
            Statement::End => write!(f, "{}", END),
            _ => Err(std::fmt::Error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expression::ArithOp;

    #[test]
    fn prints_string() {
        let mut variables: HashMap<char, i32> = HashMap::new();
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
        let mut variables: HashMap<char, i32> = HashMap::new();
        let mut lines: HashMap<i32, Statement> = HashMap::new();

        lines.insert(10, Statement::Let('A', Expression::Integer(42)));

        match lines.get(&10).expect("Error").execute(&mut variables) {
            Ok(maybe_flow) => assert!(maybe_flow.is_none()),
            Err(e) => panic!("{}", e),
        }

        assert_eq!(*variables.get(&'A').unwrap(), 42);
    }

    #[test]
    fn evaluates_condition() {
        let mut variables: HashMap<char, i32> = HashMap::new();
        let mut lines: HashMap<i32, Statement> = HashMap::new();

        variables.insert('A', 42);

        lines.insert(
            20,
            Statement::If(
                Condition::Boolean(
                    Expression::Variable('A'),
                    Relop::EQ,
                    Expression::Integer(42),
                ),
                Box::new(Statement::Let('A', Expression::Integer(69))),
            ),
        );

        match lines.get(&20).expect("Error").execute(&mut variables) {
            Ok(maybe_flow) => assert!(maybe_flow.is_none()),
            Err(e) => panic!("{}", e),
        }

        assert_eq!(*variables.get(&'A').unwrap(), 69);
    }

    #[test]
    fn evaluates_complex_expressions() {
        let mut variables: HashMap<char, i32> = HashMap::new();
        let mut lines: HashMap<i32, Statement> = HashMap::new();

        let exp = Expression::Operator(
            ArithOp::Add,
            Some(Box::new(Expression::Integer(2))),
            Some(Box::new(Expression::Operator(
                ArithOp::Multiply,
                Some(Box::new(Expression::Integer(3))),
                Some(Box::new(Expression::Integer(4))),
            ))),
        );

        lines.insert(10, Statement::Let('N', exp));

        match lines.get(&10).expect("Error").execute(&mut variables) {
            Ok(maybe_flow) => assert!(maybe_flow.is_none()),
            Err(e) => panic!("{}", e),
        }

        assert_eq!(*variables.get(&'N').unwrap(), 14);
    }

    #[test]
    fn branches_unconditionally() {
        let mut variables: HashMap<char, i32> = HashMap::new();
        let mut lines: HashMap<i32, Statement> = HashMap::new();

        variables.insert('X', 1);

        lines.insert(10, Statement::Goto(30));
        lines.insert(20, Statement::Let('X', Expression::Integer(2)));
        lines.insert(30, Statement::Let('X', Expression::Integer(3)));

        match lines.get(&10).expect("Error").execute(&mut variables) {
            Ok(maybe_flow) => match maybe_flow.unwrap() {
                ProgramSignal::Jump(f) => assert_eq!(f, 30),
                _ => panic!("Wrong type of program flow"),
            },
            Err(e) => panic!("{}", e),
        }
    }
}
