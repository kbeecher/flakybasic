use crate::statement::Statement;

pub fn update_program(program: &mut Vec<(i32, Statement)>, new_line: (i32, Statement)) {
    // The program is empty, so insert immediately.
    if program.len() == 0 {
        program.push(new_line);
        return;
    }

    // Look for a place to add it, starting from the end of the program...
    for idx in (0..program.len()).rev() {
        if new_line.0 == program[idx].0 {
            // If the line number already exists, we must check if we're
            // replacing with a new line or just removing the old one.
            match new_line.1 {
                Statement::Empty => {
                    program.remove(idx);
                }
                _ => {
                    program[idx] = (new_line.0, new_line.1);
                }
            }

            return;
        }

        if new_line.0 > program[idx].0 {
            // If the new lines's number is bigger than the current,
            // insert it in the following position.
            program.insert(idx + 1, new_line);
            return;
        }
    }

    // If we reach this point, the line number must be lower than all others.
    // Insert it at the beginning.
    program.insert(0, (new_line.0, new_line.1));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn updates_an_empty_program() {
        let mut program: Vec<(i32, Statement)> = Vec::new();
        let new_line = (10, Statement::Rem(String::from("Hello")));

        update_program(&mut program, new_line);

        assert!(program.len() == 1);
    }

    #[test]
    fn updates_with_one() {
        let mut program: Vec<(i32, Statement)> = Vec::new();
        let new_line_1 = (10, Statement::Rem(String::from("Hello")));
        let new_line_2 = (20, Statement::Rem(String::from("World")));

        update_program(&mut program, new_line_1);
        update_program(&mut program, new_line_2);

        assert_eq!(program.len(), 2);
        assert_eq!(program[0].0, 10);
        assert_eq!(program[1].0, 20);
    }

    #[test]
    fn inserts_with_many() {
        let mut program: Vec<(i32, Statement)> = Vec::new();
        let new_line_1 = (10, Statement::Rem(String::from("Hello")));
        let new_line_2 = (20, Statement::Rem(String::from("World")));
        let new_line_3 = (15, Statement::Rem(String::from(", ")));

        update_program(&mut program, new_line_1);
        update_program(&mut program, new_line_2);
        update_program(&mut program, new_line_3);

        assert_eq!(program.len(), 3);
        assert_eq!(program[0].0, 10);
        assert_eq!(program[1].0, 15);
        assert_eq!(program[2].0, 20);
    }
}
