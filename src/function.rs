use std::collections::HashMap;

use crate::{
    errors::BasicError,
    expression::{Expression, Number, eval_expression},
};

pub const INT: &str = "int";
pub const RND: &str = "rnd";

/// Evaluate a function and return the result.
///
/// # Arguments
/// * `name` - Name of the function
/// * `args` - Arguments to the function
/// * `variables` - The variable table
///
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

/// Remove any fractional part of a value and returns the integer part.
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

/// Generate a random float value in the range of 0 to 1.
fn rnd() -> Number {
    return Number::Float(rand::random_range(0.0..1.0));
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn evaluates_int() {
        let args = vec![Expression::Numeric(Number::Float(3.14))];

        match eval_function(&String::from("int"), &args, &HashMap::new()) {
            Ok(res) => {
                if let Number::Integer(i) = res {
                    assert_eq!(i, 3);
                }
            }
            Err(e) => panic!("Error: {}", e),
        }
    }

    #[test]
    fn evaluates_rnd() {
        match eval_function(&String::from("rnd"), &Vec::new(), &HashMap::new()) {
            Ok(res) => {
                if let Number::Float(n) = res {
                    assert!(n >= 0.0 && n < 1.0);
                }
            }
            Err(e) => panic!("{}", e),
        }
    }
}
