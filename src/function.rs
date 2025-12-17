use crate::{
    errors::BasicError,
    expression::{Expression, Number},
};

pub const RND: &str = "rnd";

pub fn eval_function(name: &String, _args: &Vec<Expression>) -> Result<Number, BasicError> {
    match name.as_str().trim() {
        RND => Ok(rnd()),

        _ => Err(BasicError::RuntimeError(String::from(format!(
            "Unknown function {}",
            name
        )))),
    }
}

fn rnd() -> Number {
    return Number::Float(rand::random_range(0.0..1.0));
}
