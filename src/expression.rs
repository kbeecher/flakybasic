use std::{
    collections::HashMap,
    fmt::Display,
    ops::{Add, Div, Mul, Sub},
};

use crate::{errors::BasicError, function::eval_function};

/// A numeric value, either an integer or a float.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum Number {
    Integer(i32),
    Float(f64),
}

impl Number {
    pub fn is_int(&self) -> bool {
        match self {
            Self::Integer(_) => true,
            _ => false,
        }
    }

    pub fn int_value(&self) -> Result<i32, BasicError> {
        match self {
            Self::Integer(i) => Ok(*i),
            _ => Err(BasicError::RuntimeError(String::from("Type error"))),
        }
    }
}

impl Add for Number {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match self {
            Number::Integer(self_i) => match rhs {
                Number::Integer(other_i) => Number::Integer(self_i + other_i),
                Number::Float(other_f) => Number::Float(self_i as f64 + other_f),
            },
            Number::Float(self_f) => match rhs {
                Number::Integer(other_i) => Number::Float(self_f + other_i as f64),
                Number::Float(other_f) => Number::Float(self_f + other_f),
            },
        }
    }
}

impl Sub for Number {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        match self {
            Number::Integer(self_i) => match rhs {
                Number::Integer(other_i) => Number::Integer(self_i - other_i),
                Number::Float(other_f) => Number::Float(self_i as f64 - other_f),
            },
            Number::Float(self_f) => match rhs {
                Number::Integer(other_i) => Number::Float(self_f - other_i as f64),
                Number::Float(other_f) => Number::Float(self_f - other_f),
            },
        }
    }
}

impl Mul for Number {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        match self {
            Number::Integer(self_i) => match rhs {
                Number::Integer(other_i) => Number::Integer(self_i * other_i),
                Number::Float(other_f) => Number::Float(self_i as f64 * other_f),
            },
            Number::Float(self_f) => match rhs {
                Number::Integer(other_i) => Number::Float(self_f * other_i as f64),
                Number::Float(other_f) => Number::Float(self_f * other_f),
            },
        }
    }
}

impl Div for Number {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        match self {
            Number::Integer(self_i) => match rhs {
                Number::Integer(other_i) => Number::Integer(self_i / other_i),
                Number::Float(other_f) => Number::Float(self_i as f64 / other_f),
            },
            Number::Float(self_f) => match rhs {
                Number::Integer(other_i) => Number::Float(self_f / other_i as f64),
                Number::Float(other_f) => Number::Float(self_f / other_f),
            },
        }
    }
}

impl Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Number::Integer(n) => write!(f, "{}", n),
            Number::Float(n) => write!(f, "{}", n),
        }
    }
}

/// An expression in the language.
#[derive(Clone, Debug)]
pub enum Expression {
    String(String),
    Numeric(Number),
    Variable(char),

    /// An operator is a recursive binary tree where non-leaf nodes
    /// are operators.
    Operator(ArithOp, Option<Box<Expression>>, Option<Box<Expression>>),

    /// A function call with optional arguments
    Function(String, Vec<Expression>),
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
            Expression::Numeric(n) => write!(f, "{}", n),
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
            Expression::Function(name, args) => {
                let mut output = String::new();
                let first = true;

                output.push_str(&format!("{}(", name));

                for a in args.iter() {
                    if first {
                        output.push_str(&format!("{}", a));
                    } else {
                        output.push_str(&format!(", {}", a));
                    }
                }

                output.push_str(")");
                write!(f, "{}", output)
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
#[derive(Copy, Clone, Debug)]
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
    variables: &HashMap<char, Number>,
) -> Result<Number, BasicError> {
    match root {
        Expression::Numeric(n) => return Ok(n),
        Expression::Variable(c) => match variables.get(&c) {
            Some(v) => return Ok(v.clone()),
            _ => {
                return Err(BasicError::RuntimeError(String::from(format!(
                    "Unknown variable {}",
                    c
                ))));
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
        Expression::Function(name, args) => {
            return eval_function(&name, &args, variables);
        }
        _ => panic!("Invalid type in expression"),
    }
}
