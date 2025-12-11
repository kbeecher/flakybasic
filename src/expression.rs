use std::{collections::HashMap, fmt::Display};

use crate::errors::BasicError;

/// An expression in the language.
#[derive(Clone)]
pub enum Expression {
    String(String),
    Integer(i32),
    Variable(char),

    /// An operator is a recursive binary tree where non-leaf nodes
    /// are operators.
    Operator(ArithOp, Option<Box<Expression>>, Option<Box<Expression>>),
}

fn override_precedence(op: &ArithOp, exp: &Expression) -> bool {
    match op {
        ArithOp::Multiply | ArithOp::Divide => match exp {
            Expression::Operator(other_op, _, _) => match other_op {
                ArithOp::Add | ArithOp::Subtract => true,
                _ => false,
            },
            _ => false,
        },
        _ => false,
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Integer(i) => write!(f, "{}", i),
            Expression::Variable(c) => write!(f, "{}", c),
            Expression::String(s) => write!(f, "\"{}\"", s),
            Expression::Operator(op, l_exp, r_exp) => {
                if override_precedence(op, &l_exp.as_ref().unwrap()) {
                    let _ = write!(f, "(");
                }

                match write!(f, "{}", l_exp.as_ref().expect("Error")) {
                    Ok(_) => {}
                    Err(e) => return Err(e),
                }

                if override_precedence(op, &l_exp.as_ref().unwrap()) {
                    let _ = write!(f, ")");
                }

                match write!(f, "{}", op) {
                    Ok(_) => {}
                    Err(e) => return Err(e),
                }

                if override_precedence(op, &r_exp.as_ref().unwrap()) {
                    let _ = write!(f, "(");
                }

                match write!(f, "{}", r_exp.as_ref().expect("Error")) {
                    Ok(_) => {}
                    Err(e) => return Err(e),
                }

                if override_precedence(op, &r_exp.as_ref().unwrap()) {
                    let _ = write!(f, ")");
                }

                return Ok(());
            }
        }
    }
}

/// Relative operators.
#[derive(Copy, Clone)]
pub enum Relop {
    EQ,
    NEQ,
    LT,
    LTE,
    GT,
    GTE,
}

impl Display for Relop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Relop::EQ => write!(f, "{}", "="),
            Relop::NEQ => write!(f, "{}", "<>"),
            Relop::LT => write!(f, "{}", "<"),
            Relop::LTE => write!(f, "{}", "<="),
            Relop::GT => write!(f, "{}", ">"),
            Relop::GTE => write!(f, "{}", ">="),
        }
    }
}

// TODO Merge with Expression type?
pub enum Condition {
    Boolean(Expression, Relop, Expression),
}

impl Display for Condition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Condition::Boolean(l_exp, relop, r_exp) => write!(f, "{}{}{}", l_exp, relop, r_exp),
        }
    }
}

/// Arithmetic operators.
#[derive(Copy, Clone)]
pub enum ArithOp {
    Add,
    Subtract,
    Multiply,
    Divide,
}

impl Display for ArithOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArithOp::Add => write!(f, "{}", '+'),
            ArithOp::Subtract => write!(f, "{}", '-'),
            ArithOp::Multiply => write!(f, "{}", '*'),
            ArithOp::Divide => write!(f, "{}", '/'),
        }
    }
}

/// Evaluate an expression and reduce it to a single integer value.
pub fn eval_expression(
    root: Expression,
    variables: &HashMap<char, i32>,
) -> Result<i32, BasicError> {
    match root {
        Expression::Integer(i) => return Ok(i),
        Expression::Variable(c) => match variables.get(&c) {
            Some(v) => return Ok(*v),
            _ => {
                return Err(BasicError::RuntimeError(String::from(
                    "Unknown variable {}",
                )));
            }
        },
        Expression::Operator(op, l_exp, r_exp) => match op {
            ArithOp::Add => {
                return Ok(eval_expression(*l_exp.unwrap(), variables)?
                    + eval_expression(*r_exp.unwrap(), variables)?);
            }
            ArithOp::Subtract => {
                return Ok(eval_expression(*l_exp.unwrap(), variables)?
                    - eval_expression(*r_exp.unwrap(), variables)?);
            }
            ArithOp::Multiply => {
                return Ok(eval_expression(*l_exp.unwrap(), variables)?
                    * eval_expression(*r_exp.unwrap(), variables)?);
            }
            ArithOp::Divide => {
                return Ok(eval_expression(*l_exp.unwrap(), variables)?
                    / eval_expression(*r_exp.unwrap(), variables)?);
            }
        },
        _ => panic!("Invalid type in expression"),
    }
}
