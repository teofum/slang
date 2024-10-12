use crate::error::ParseError;
use crate::prologue::PROLOGUE;
use fancy_regex::{Captures, Regex};
use rand::Rng;
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
    Nop,
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

        let nop_regex: Regex = Regex::new(r"^nop$").unwrap();
        if let Some(_) = nop_regex.captures(instruction)? {
            return Ok(Some(Instruction::Nop));
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

        // Read source file and append its lines to prologue
        let reader = BufReader::new(file);
        let lines: Vec<_> = PROLOGUE.lines()
            .map(|str| str.to_string())
            .chain(reader.lines().flatten())
            .enumerate()
            .collect();

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
        // Find a label and add it to the program's list of labels
        let instruction = Self::find_label(
            instruction,
            self.instructions.len(),
            &mut self.labels,
            line_num,
        )?.trim();

        // Match an instruction
        if let Some(instruction) = Instruction::parse(instruction, line_num)? {
            self.instructions.push(instruction);
            return Ok(());
        }

        // Match macros
        for m in &self.macros {
            if let Some(caps) = m.pattern.captures(instruction)? {
                Self::expand_macro(
                    &self.macros,
                    m,
                    &mut self.instructions,
                    &mut self.labels,
                    &mut self.max_temp_var,
                    &caps,
                    line_num,
                )?;
                return Ok(());
            }
        }

        Err(ParseError::boxed(
            &format!("Expression {} is not a valid instruction", instruction),
            line_num)
        )
    }

    fn find_label<'a>(
        instruction: &'a str,
        instruction_number: usize,
        labels: &mut HashMap<String, usize>,
        line_num: usize,
    ) -> Result<&'a str, Box<dyn Error>> {
        let label_regex: Regex = Regex::new(r"^\[(\w+)]").unwrap();
        match label_regex.captures(instruction)? {
            Some(caps) => {
                let full = &caps[0];
                let label_name = &caps[1];

                if labels.contains_key(label_name) {
                    return Err(ParseError::boxed(&format!("Redefined label {}", label_name), line_num));
                }

                labels.insert(label_name.to_owned(), instruction_number);
                Ok(instruction.strip_prefix(full).unwrap())
            }
            None => Ok(instruction)
        }
    }

    fn expand_macro(
        macros: &Vec<Macro>,
        m: &Macro,
        instructions: &mut Vec<Instruction>,
        labels: &mut HashMap<String, usize>,
        max_temp_var: &mut usize,
        caps: &Captures,
        line_num: usize,
    ) -> Result<(), Box<dyn Error>> {
        let auto_var_regex: Regex = Regex::new(r"\$(\w+)").unwrap();
        let mut auto_vars = HashMap::new();

        let mut rng = rand::thread_rng();
        let uid: u64 = rng.gen();
        for instruction in &m.instructions {
            // Expand % token
            let instruction = instruction.replace("%", &format!("MACRO_{:x}_", uid));

            // Find labels
            let instruction = Self::find_label(
                &instruction,
                instructions.len(),
                labels,
                line_num,
            )?.trim();

            // Perform macro replacements
            let mut instruction = instruction.to_string();
            for (pattern, cap) in &m.replacements {
                instruction = instruction.replace(pattern, &caps[*cap + 1]);
            }

            // Replace automatic variables
            let instruction = auto_var_regex.replace_all(&instruction, |caps: &Captures| {
                let var_name = caps[1].to_string();
                let var_num = auto_vars.entry(var_name).or_insert_with(|| {
                    *max_temp_var += 1;
                    *max_temp_var
                });

                format!("z{}", var_num)
            });

            if let Some(instruction) = Instruction::parse(&instruction, line_num)? {
                instructions.push(instruction);
            } else {
                for m in macros {
                    if let Some(caps) = m.pattern.captures(&instruction)? {
                        Self::expand_macro(
                            macros,
                            m,
                            instructions,
                            labels,
                            max_temp_var,
                            &caps,
                            line_num,
                        )?;
                        break;
                    }
                }
            }
        }

        Ok(())
    }
}