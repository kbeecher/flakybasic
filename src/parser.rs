use std::char;

use crate::{
    errors::BasicError,
    expression::{ArithOp, Condition, Expression, Number, Relop},
    statement::Statement,
};

// Keywords as they appear in source code.
pub const REM: &str = "rem";
pub const PRINT: &str = "print";
pub const LET: &str = "let";
pub const GOTO: &str = "goto";
pub const IF: &str = "if";
pub const THEN: &str = "then";
pub const INPUT: &str = "input";
pub const GOSUB: &str = "gosub";
pub const RETURN: &str = "return";
pub const FOR: &str = "for";
pub const TO: &str = "to";
pub const STEP: &str = "step";
pub const NEXT: &str = "next";
pub const LIST: &str = "list";
pub const RUN: &str = "run";
pub const LOAD: &str = "load";
pub const SAVE: &str = "save";
pub const CLEAR: &str = "clear";
pub const END: &str = "end";

/// A structure used to track the parsing of a single statement.
pub struct SourceReader {
    /// The source line
    line: String,

    /// The index of the line's current character being examinined
    idx: usize,

    /// The depth into any expression in a statement. Must be 0 for a
    /// balanced expression.
    exp_depth: usize,
}

impl SourceReader {
    pub fn new(src_line: String) -> SourceReader {
        SourceReader {
            line: src_line,
            idx: 0,
            exp_depth: 0,
        }
    }

    /// Get a value that can only be a whole number (e.g. line number)
    pub fn get_integer(&mut self) -> Result<i32, BasicError> {
        self.skip_ws();
        let start_at = self.idx;

        while self.is_digit() {
            self.next();
        }

        let end_at = self.idx;
        self.skip_ws();

        match self.line[start_at..end_at].parse::<i32>() {
            Ok(i) => Ok(i),
            Err(_) => Err(BasicError::SyntaxError(String::from(
                "Error reading number",
            ))),
        }
    }

    /// Get an integer or float at the current point in the line.
    pub fn get_number(&mut self) -> Result<Number, BasicError> {
        self.skip_ws();
        let start_at = self.idx;

        while self.is_digit() || self.ch() == '.' {
            self.next();
        }

        let end_at = self.idx;
        self.skip_ws();

        match self.line[start_at..end_at].parse::<i32>() {
            Ok(i) => Ok(Number::Integer(i)),
            Err(_) => match self.line[start_at..end_at].parse::<f64>() {
                Ok(f) => Ok(Number::Float(f)),
                Err(_) => Err(BasicError::SyntaxError(String::from(
                    "Error reading number",
                ))),
            },
        }
    }

    /// Get a keyword at the current point in the line. A keyword is made
    /// up purely of alphabetic characters.
    fn get_keyword(&mut self) -> &str {
        self.skip_ws();
        let start_at = self.idx;

        while self.is_alpha() {
            self.next();
        }

        let end_at = self.idx;
        self.skip_ws();

        &self.line[start_at..end_at]
    }

    /// Get a string at the current point in the line. A string begins and
    /// ends with the " delimiter.
    fn get_string(&mut self) -> Result<&str, BasicError> {
        self.skip_ws();

        if self.ch() != '"' {
            return Err(BasicError::SyntaxError(String::from("Expected \"")));
        }

        // Skip quote mark
        self.next();

        let start_at = self.idx;

        while self.ch() != '"' {
            // Have we prematurely reached the end of the line?
            if self.at_end() || self.ch() == '\n' {
                return Err(BasicError::SyntaxError(String::from("Unterminated string")));
            }

            self.next();
        }

        // Current char is closing quote.
        let end_at = self.idx;

        // Skip over quote and return the string contents
        self.next();
        self.skip_ws();

        Ok(&self.line[start_at..end_at])
    }

    /// Return the text between the current point in the line and its end.
    fn get_text(&mut self) -> Result<String, BasicError> {
        self.skip_ws();
        let text = self.line[self.idx..self.line.len()].to_string();
        self.idx = self.line.len();

        Ok(text)
    }

    /// Get a single character at the current point in the line.
    fn get_char(&mut self) -> Result<char, BasicError> {
        self.skip_ws();

        if self.is_alpha() {
            let c = self.ch();
            self.next();
            return Ok(c);
        } else {
            Err(BasicError::SyntaxError(String::from(
                "Failed to match a character",
            )))
        }
    }

    /// Get a relative oeprator at the current point in the line.
    fn get_relop(&mut self) -> Result<Relop, BasicError> {
        self.skip_ws();
        let mut relop = String::new();

        if self.ch() == '=' {
            relop.push(self.ch());
            self.next();
        } else if self.ch() == '<' {
            relop.push(self.ch());
            self.next();

            if self.ch() == '=' {
                relop.push(self.ch());
                self.next();
            } else if self.ch() == '>' {
                relop.push(self.ch());
                self.next();
            }
        } else if self.ch() == '>' {
            relop.push(self.ch());
            self.next();

            if self.ch() == '=' {
                relop.push(self.ch());
                self.next();
            }
        }

        self.skip_ws();

        match relop.as_str() {
            "=" => Ok(Relop::EQ),
            "<" => Ok(Relop::LT),
            ">" => Ok(Relop::GT),
            "<=" => Ok(Relop::LTE),
            ">=" => Ok(Relop::GTE),
            "<>" => Ok(Relop::NEQ),
            _ => Err(BasicError::SyntaxError(String::from(
                "Relative operator not recognised",
            ))),
        }
    }

    /// Get a numeric expression at the current point in the line.
    fn get_expression(&mut self) -> Result<Expression, BasicError> {
        let mut root = self.get_term()?;

        while !self.at_end() {
            self.skip_ws();

            if self.is_addop() {
                if self.ch() == '+' {
                    self.skip_token(String::from("+"));
                    let term = self.get_term()?;
                    root = self.make_subtree(ArithOp::Add, root, term);
                } else if self.ch() == '-' {
                    self.skip_token(String::from('-'));
                    let term = self.get_term()?;
                    root = self.make_subtree(ArithOp::Subtract, root, term);
                }
            } else if self.ch() == ')' && self.exp_depth > 0 {
                self.skip_token(String::from(")"));
                self.exit_subexp();
                break;
            } else {
                break;
            }
        }

        return Ok(root);
    }

    fn get_term(&mut self) -> Result<Expression, BasicError> {
        let mut root = self.get_factor()?;

        while !self.at_end() {
            self.skip_ws();

            if self.is_mulop() {
                if self.ch() == '*' {
                    self.skip_token(String::from("*"));
                    let factor = self.get_factor()?;
                    root = self.make_subtree(ArithOp::Multiply, root, factor);
                } else if self.ch() == '/' {
                    self.skip_token(String::from("/"));
                    let factor = self.get_factor()?;
                    root = self.make_subtree(ArithOp::Divide, root, factor);
                }
            } else {
                break;
            }
        }

        return Ok(root);
    }

    fn get_factor(&mut self) -> Result<Expression, BasicError> {
        if self.at_end() {
            return Err(BasicError::SyntaxError(String::from(
                "Unexpected end of line",
            )));
        }

        // Is it a variable or function call?
        if self.is_alpha() {
            let mut identifier = String::new();

            // Get first letter
            identifier.push_str(&self.ch().to_string());

            // Any more alphabetic characters?
            self.next();
            if self.is_alpha() {
                // Yes, so it must be a function call.
                while self.is_alpha() {
                    identifier.push_str(&self.get_char()?.to_string());
                }

                // Get args
                let mut args: Vec<Expression> = Vec::new();
                self.skip_token("(".to_string());

                while self.ch() != ')' {
                    args.push(self.get_expression()?);
                    self.skip_ws();

                    if self.ch() != ',' {
                        self.skip_token(",".to_string());
                    }
                }

                self.skip_token(")".to_string());

                return Ok(Expression::Function(identifier, args));
            }

            // If we reach here, there was only one letter, so it's a variable
            return Ok(Expression::Variable(
                identifier.chars().nth(0).expect("Error reading identifier"),
            ));
        }

        // Is it a (possibly negative) number?
        let mut adjust: Number = Number::Integer(1);
        if self.ch() == '-' {
            self.skip_token(String::from("-"));
            adjust = Number::Integer(-1);
        }

        if self.is_digit() {
            return Ok(Expression::Numeric(self.get_number()? * adjust));
        }

        // Is it a subexpression?
        if self.ch() == '(' {
            self.skip_token(String::from("("));
            self.enter_subexp();

            return Ok(self.get_expression()?);
        }

        Err(BasicError::SyntaxError(String::from("Error in expression")))
    }

    /// Join two child expressions to a parent operator to produce a
    /// binary subtree.
    fn make_subtree(
        &self,
        operator: ArithOp,
        l_child: Expression,
        r_child: Expression,
    ) -> Expression {
        Expression::Operator(operator, Some(Box::new(l_child)), Some(Box::new(r_child)))
    }

    /// Get the character at the current point in the line.
    fn ch(&self) -> char {
        match self.line.chars().nth(self.idx) {
            Some(c) => return c,
            None => return '\0',
        }
    }

    /// Have we reached the end of line?
    fn at_end(&self) -> bool {
        self.idx >= self.line.len()
    }

    /// Move to the next character in the line.
    fn next(&mut self) {
        self.idx += 1;
    }

    /// Skip whitespace until reaching either the end of the line or
    /// non-whitespace character.
    pub fn skip_ws(&mut self) {
        while !self.at_end() && self.is_space() {
            self.next();
        }
    }

    /// Skip past the specified token. If the specified token isn't found
    /// at the current point in the line, generate an error.
    fn skip_token(&mut self, token: String) -> Option<BasicError> {
        // TODO Change token to &str?
        self.skip_ws();

        for c in token.chars() {
            if self.ch() != c {
                return Some(BasicError::SyntaxError(format!("Expected {}", token)));
            }
            self.next();
        }

        self.skip_ws();

        None
    }

    fn enter_subexp(&mut self) {
        self.exp_depth += 1;
    }

    fn exit_subexp(&mut self) -> Option<BasicError> {
        if self.exp_depth > 0 {
            self.exp_depth -= 1;
        } else {
            return Some(BasicError::SyntaxError(String::from("Too many ')'")));
        }

        None
    }

    fn is_space(&self) -> bool {
        self.ch().is_whitespace()
    }

    pub fn is_digit(&self) -> bool {
        self.ch().is_digit(10)
    }

    fn is_alpha(&self) -> bool {
        self.ch().is_alphabetic()
    }

    fn is_addop(&self) -> bool {
        self.ch() == '+' || self.ch() == '-'
    }

    fn is_mulop(&self) -> bool {
        self.ch() == '*' || self.ch() == '/'
    }

    /// Compile the current line into a Statement.
    pub fn build_statement(&mut self) -> Result<Statement, BasicError> {
        let keyword = self.get_keyword();

        let statement = match keyword {
            REM => {
                let comment = self.get_text()?;
                Ok(Statement::Rem(comment))
            }

            PRINT => {
                let mut another = true;
                let mut args: Vec<Expression> = Vec::new();

                while another {
                    another = false;

                    if self.ch() == '"' {
                        let msg = self.get_string()?.to_string();
                        args.push(Expression::String(msg));
                    } else if !self.at_end() {
                        args.push(self.get_expression()?);
                    }

                    self.skip_ws();

                    // More?
                    if self.ch() == ',' {
                        another = true;
                        self.skip_token(String::from(","));
                        self.skip_ws();
                    }
                }

                Ok(Statement::Print(args))
            }

            LET => {
                let var_name = self.get_char()?;
                self.skip_token(String::from("="));
                let exp = Ok(self.get_expression()?);

                Ok(Statement::Let(var_name, exp?))
            }

            IF => {
                let l_exp = self.get_expression()?;
                let relop = self.get_relop()?;
                let r_exp = self.get_expression()?;
                self.skip_token(String::from(THEN));

                // Recursive call to build the consequent statement.
                let sub = self.build_statement()?;

                Ok(Statement::If(
                    Condition::Boolean(l_exp, relop, r_exp),
                    Box::new(sub),
                ))
            }

            GOTO => Ok(Statement::Goto(self.get_integer()?)),

            INPUT => Ok(Statement::Input(self.get_char()?)),

            GOSUB => Ok(Statement::Gosub(self.get_integer()?)),

            RETURN => Ok(Statement::Return),

            FOR => {
                let var = self.get_char()?;
                self.skip_token(String::from("="));
                let start_val = self.get_expression()?;
                self.skip_token(TO.to_string());
                let end_val = self.get_expression()?;
                self.skip_ws();

                let step_val = match self.at_end() {
                    true => None,
                    false => {
                        self.skip_token(STEP.to_string());
                        self.skip_ws();
                        Some(self.get_expression()?)
                    }
                };

                Ok(Statement::For(var, start_val, end_val, step_val))
            }

            NEXT => Ok(Statement::Next),

            LIST => Ok(Statement::List),

            RUN => Ok(Statement::Run),

            LOAD => Ok(Statement::Load(self.get_string()?.to_string())),

            SAVE => Ok(Statement::Save(self.get_string()?.to_string())),

            CLEAR => Ok(Statement::Clear),

            END => Ok(Statement::End),

            _ => {
                match keyword.len() {
                    0 => {
                        // Did it fail because the line is empty?
                        if self.at_end() {
                            return Ok(Statement::Empty);
                        }
                        return Err(BasicError::SyntaxError(String::from("Unknown keyword")));
                    }
                    1 => {
                        // Is it a let statement without let?
                        let var_name = keyword.chars().nth(0).expect("Error getting keyword");
                        self.skip_token(String::from("="));
                        let exp = self.get_expression()?;

                        return Ok(Statement::Let(var_name, exp));
                    }
                    _ => {
                        // Otherwise, whatever is there isn't a recognisable keyword
                        return Err(BasicError::SyntaxError(String::from("Unknown keyword")));
                    }
                }
            }
        };

        // Post compile checks
        if !self.at_end() {
            return Err(BasicError::SyntaxError(String::from("Unexpected token")));
        }

        if !self.exp_depth == 0 {
            return Err(BasicError::SyntaxError(String::from("Invalid expression")));
        }

        return statement;
    }
}
