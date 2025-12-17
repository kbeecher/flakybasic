use std::collections::HashMap;

use crate::{
    errors::BasicError,
    expression::{Expression, Number, eval_expression},
};

pub const INT: &str = "int";
pub const RND: &str = "rnd";

pub fn eval_function(
    name: &String,
    args: &Vec<Expression>,
    variables: &HashMap<char, Number>,
) -> Result<Number, BasicError> {
    match name.as_str().trim() {
        RND => Ok(rnd()),

        INT => int(args, variables),

        _ => Err(BasicError::RuntimeError(String::from(format!(
            "Unknown identifier {}",
            name
        )))),
    }
}

fn int(args: &Vec<Expression>, variables: &HashMap<char, Number>) -> Result<Number, BasicError> {
    if args.len() != 1 {
        return Err(BasicError::RuntimeError(String::from(
            "Incorrect number of arguments to function int",
        )));
    }

    let arg = args.iter().nth(0).expect("Error evaluating function int");
    let val = eval_expression(arg.clone(), &variables)?;

    match val {
        Number::Integer(i) => Ok(Number::Integer(i)),
        Number::Float(f) => Ok(Number::Integer(f as i32)),
    }
}

fn rnd() -> Number {
    return Number::Float(rand::random_range(0.0..1.0));
}
