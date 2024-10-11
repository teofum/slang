use crate::error::ParseError;
use fancy_regex::Regex;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub enum Variable {
    X(usize),
    Y,
    Z(usize),
}

impl Variable {
    pub fn parse(var: &str, line_num: usize) -> Result<Self, Box<dyn Error>> {
        match var.chars().next() {
            Some('x') => Ok(Variable::X(var[1..].parse()?)),
            Some('y') => Ok(Variable::Y),
            Some('z') => Ok(Variable::Z(var[1..].parse()?)),
            _ => Err(ParseError::boxed("Invalid variable name", line_num))
        }
    }
}

pub enum Instruction {
    Increment { var: Variable },
    Decrement { var: Variable },
    JumpNonZero { var: Variable, to: String },
}

pub struct Program {
    pub instructions: Vec<Instruction>,
    pub labels: HashMap<String, usize>,
}

impl Program {
    pub fn from_file(file: &File) -> Result<Self, Box<dyn Error>> {
        let reader = BufReader::new(file);
        let mut program = Program { instructions: Vec::new(), labels: HashMap::new() };

        for (line_num, line) in reader.lines().flatten().enumerate() {
            if line.len() == 0 || line.starts_with('#') {
                continue; // Skip empty lines and comments
            }

            if line.starts_with('@') {
                // Process directives:
                // @def [macro] / @end
                continue;
            }

            program.parse_line(&line, line_num)?;
        }

        Ok(program)
    }

    fn parse_line(&mut self, instruction: &str, line_num: usize) -> Result<(), Box<dyn Error>> {
        let instruction_number = self.instructions.len();
        let mut instruction = instruction;

        // Find a label and add it to the program's list of labels
        let label_regex = Regex::new(r"^\[([a-zA-Z0-9_]+)]").unwrap();
        if let Some(caps) = label_regex.captures(instruction)? {
            let full = &caps[0];
            let label_name = &caps[1];

            if self.labels.contains_key(label_name) {
                return Err(ParseError::boxed(&format!("Redefined label {}", label_name), line_num));
            }

            self.labels.insert(label_name.to_owned(), instruction_number);
            instruction = instruction.strip_prefix(full).unwrap();
        }

        let instruction = instruction.trim();

        // Match an instruction
        let inc_regex = Regex::new(r"^(y|[xz]\d+) <- \1 \+ 1$").unwrap();
        if let Some(caps) = inc_regex.captures(instruction)? {
            let instruction = Instruction::Increment { var: Variable::parse(&caps[1], line_num)? };
            self.instructions.push(instruction);
            return Ok(());
        }

        let dec_regex = Regex::new(r"^(y|[xz]\d) <- (\1) - 1$").unwrap();
        if let Some(caps) = dec_regex.captures(instruction)? {
            let instruction = Instruction::Decrement { var: Variable::parse(&caps[1], line_num)? };
            self.instructions.push(instruction);
            return Ok(());
        }

        let jnz_regex = Regex::new(r"^if (y|[xz]\d) != 0 goto ([a-zA-Z0-9_]+)$").unwrap();
        let jnz_alt_regex = Regex::new(r"^jnz (y|[xz]\d) ([a-zA-Z0-9_]+)$").unwrap();
        if let Some(caps) = jnz_regex.captures(instruction)?.or(jnz_alt_regex.captures(instruction)?) {
            let instruction = Instruction::JumpNonZero {
                var: Variable::parse(&caps[1], line_num)?,
                to: caps[2].to_owned(),
            };
            self.instructions.push(instruction);
            return Ok(());
        }

        Err(ParseError::boxed(
            &format!("Expression {} is not a valid instruction", instruction),
            line_num)
        )
    }
}