use crate::error::ParseError;
use fancy_regex::{Captures, Regex};
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

impl Instruction {
    pub fn parse(instruction: &str, line_num: usize) -> Result<Option<Self>, Box<dyn Error>> {
        let inc_regex: Regex = Regex::new(r"^(y|[xz]\d+) <- \1 \+ 1$").unwrap();
        if let Some(caps) = inc_regex.captures(instruction)? {
            let instruction = Instruction::Increment { var: Variable::parse(&caps[1], line_num)? };
            return Ok(Some(instruction));
        }

        let dec_regex: Regex = Regex::new(r"^(y|[xz]\d) <- (\1) - 1$").unwrap();
        if let Some(caps) = dec_regex.captures(instruction)? {
            let instruction = Instruction::Decrement { var: Variable::parse(&caps[1], line_num)? };
            return Ok(Some(instruction));
        }

        let jnz_regex: Regex = Regex::new(r"^if (y|[xz]\d) != 0 goto (\w+)$").unwrap();
        let jnz_alt_regex: Regex = Regex::new(r"^jnz (y|[xz]\d) (\w+)$").unwrap();
        if let Some(caps) = jnz_regex.captures(instruction)?.or(jnz_alt_regex.captures(instruction)?) {
            let instruction = Instruction::JumpNonZero {
                var: Variable::parse(&caps[1], line_num)?,
                to: caps[2].to_owned(),
            };
            return Ok(Some(instruction));
        }

        Ok(None)
    }
}

pub struct Macro {
    pub pattern: Regex,
    pub replacements: HashMap<String, usize>,
    pub instructions: Vec<String>,
}

impl Macro {
    pub fn parse(def: &str) -> Self {
        let macro_def_regex: Regex = Regex::new(r"\{(\w+)}").unwrap();
        let pattern = macro_def_regex.replace_all(def, r"(\w+)");
        let pattern = Regex::new(&format!("^{}$", pattern)).unwrap();

        let mut replacements = HashMap::new();
        for (n, caps) in macro_def_regex.captures_iter(def).flatten().enumerate() {
            replacements.insert(caps[1].to_string(), n);
        }

        Macro { pattern, replacements, instructions: Vec::new() }
    }
}

pub struct Program {
    pub instructions: Vec<Instruction>,
    pub labels: HashMap<String, usize>,
    pub macros: Vec<Macro>,
    max_temp_var: usize,
}

impl Program {
    pub fn from_file(file: &File) -> Result<Self, Box<dyn Error>> {
        let mut program = Program {
            instructions: Vec::new(),
            labels: HashMap::new(),
            macros: Vec::new(),
            max_temp_var: 0,
        };
        let mut current_macro: Option<Box<Macro>> = None;

        let reader = BufReader::new(file);
        let lines: Vec<_> = reader.lines().flatten().enumerate().collect();

        // Variable counting pre-pass
        let var_regex: Regex = Regex::new(r"\bz(\d+)\b").unwrap();
        for (_, line) in &lines {
            program.max_temp_var = var_regex.captures_iter(&line).flatten()
                .map(|caps| caps[1].parse::<usize>().unwrap())
                .fold(program.max_temp_var, usize::max);
        }

        // reader.seek(SeekFrom::Start(0))?;
        for (line_num, line) in lines {
            if line.len() == 0 || line.starts_with('#') {
                continue; // Skip empty lines and comments
            }

            if line.starts_with('@') {
                // Process directives:
                if line.starts_with("@def") {
                    if let Some(_) = current_macro {
                        return Err(ParseError::boxed("Unexpected nested @def directive", line_num));
                    } else {
                        current_macro = Some(Box::new(Macro::parse(&line[4..].trim())));
                    }
                } else if line.starts_with("@end") {
                    match current_macro {
                        Some(boxed_macro) => {
                            program.macros.push(*boxed_macro);
                            current_macro = None;
                        }
                        _ => return Err(ParseError::boxed("Unexpected @end directive", line_num)),
                    }
                } else {
                    return Err(ParseError::boxed("Unknown directive", line_num));
                }
                continue;
            }

            if let Some(current_macro) = &mut current_macro {
                current_macro.instructions.push(line.to_string());
            } else {
                program.parse_line(&line, line_num)?;
            }
        }

        Ok(program)
    }

    fn parse_line(&mut self, instruction: &str, line_num: usize) -> Result<(), Box<dyn Error>> {
        let instruction_number = self.instructions.len();
        let mut instruction = instruction;

        // Find a label and add it to the program's list of labels
        let label_regex: Regex = Regex::new(r"^\[(\w+)]").unwrap();
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
        if let Some(instruction) = Instruction::parse(instruction, line_num)? {
            self.instructions.push(instruction);
            return Ok(());
        }

        // Match macros
        for i in 0..self.macros.len() {
            if let Some(caps) = self.macros[i].pattern.captures(instruction)? {
                self.expand_macro(i, &caps, line_num)?;
                return Ok(());
            }
        }

        Err(ParseError::boxed(
            &format!("Expression {} is not a valid instruction", instruction),
            line_num)
        )
    }

    fn expand_macro(&mut self, macro_idx: usize, caps: &Captures, line_num: usize) -> Result<(), Box<dyn Error>> {
        let auto_var_regex: Regex = Regex::new(r"\$(\w+)").unwrap();
        let mut auto_vars = HashMap::new();

        let m = &self.macros[macro_idx];
        for instruction in &m.instructions {
            // Perform macro replacements
            let mut replaced = instruction.to_string();
            for (pattern, cap) in &m.replacements {
                replaced = replaced.replace(pattern, &caps[*cap + 1]);
            }

            // Replace automatic variables
            let replaced = auto_var_regex.replace_all(&replaced, |caps: &Captures| {
                let var_name = caps[1].to_string();
                let var_num = auto_vars.entry(var_name).or_insert_with(|| {
                    self.max_temp_var += 1;
                    self.max_temp_var
                });

                format!("z{}", var_num)
            });

            if let Some(instruction) = Instruction::parse(&replaced, line_num)? {
                self.instructions.push(instruction);
            }
        }

        Ok(())
    }
}