use crate::error::ParseError;
use crate::prologue::PROLOGUE;
use fancy_regex::{Captures, Regex};
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{BufRead, BufReader};

// =================================================================================================
// Variables
// =================================================================================================

#[derive(Debug)]
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

impl Display for Variable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Variable::X(n) => write!(f, "x{}", n),
            Variable::Z(n) => write!(f, "z{}", n),
            Variable::Y => write!(f, "y"),
        }
    }
}

// =================================================================================================
// Labels
// =================================================================================================

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Label(usize);

impl Label {
    pub fn new(group: usize, number: usize) -> Self {
        Label(number * 5 + group)
    }

    pub fn parse(label: &str, line_num: usize) -> Result<Self, Box<dyn Error>> {
        let c = label.chars().next();
        match c {
            Some(c @ 'A'..='E') => {
                let number = label[1..].parse::<usize>()?;
                let group = c as usize - 'A' as usize;
                Ok(Label(number * 5 + group))
            }
            _ => Err(ParseError::boxed("Invalid label name", line_num))
        }
    }
}

impl Display for Label {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let number = self.0 / 5;
        let group = char::from_u32('A' as u32 + (self.0 % 5) as u32).unwrap();
        write!(f, "{}{}", group, number)
    }
}

// =================================================================================================
// Instructions
// =================================================================================================

pub enum Instruction {
    Increment { var: Variable },
    Decrement { var: Variable },
    JumpNonZero { var: Variable, to: Label },
    Nop,
    Print { var: Variable },
    State,
}

impl Instruction {
    pub fn parse(instruction: &str, line_num: usize) -> Result<Option<Self>, Box<dyn Error>> {
        let inc_regex: Regex = Regex::new(r"^(y|[xz]\d+) <- \1 \+ 1$").unwrap();
        if let Some(caps) = inc_regex.captures(instruction)? {
            let instruction = Instruction::Increment { var: Variable::parse(&caps[1], line_num)? };
            return Ok(Some(instruction));
        }

        let dec_regex: Regex = Regex::new(r"^(y|[xz]\d+) <- (\1) - 1$").unwrap();
        if let Some(caps) = dec_regex.captures(instruction)? {
            let instruction = Instruction::Decrement { var: Variable::parse(&caps[1], line_num)? };
            return Ok(Some(instruction));
        }

        let jnz_regex: Regex = Regex::new(r"^if (y|[xz]\d+) != 0 goto (\w+)$").unwrap();
        if let Some(caps) = jnz_regex.captures(instruction)? {
            let instruction = Instruction::JumpNonZero {
                var: Variable::parse(&caps[1], line_num)?,
                to: Label::parse(&caps[2], line_num)?,
            };
            return Ok(Some(instruction));
        }

        let nop_regex: Regex = Regex::new(r"^nop$").unwrap();
        if nop_regex.captures(instruction)?.is_some() {
            return Ok(Some(Instruction::Nop));
        }

        let print_regex: Regex = Regex::new(r"^print (y|[xz]\d+)$").unwrap();
        if let Some(caps) = print_regex.captures(instruction)? {
            let instruction = Instruction::Print { var: Variable::parse(&caps[1], line_num)? };
            return Ok(Some(instruction));
        }

        let state_regex: Regex = Regex::new(r"^state$").unwrap();
        if state_regex.captures(instruction)?.is_some() {
            return Ok(Some(Instruction::State));
        }

        Ok(None)
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::Increment { var } => write!(f, "{} <- {0} + 1", var),
            Instruction::Decrement { var } => write!(f, "{} <- {0} - 1", var),
            Instruction::JumpNonZero { var, to } => write!(f, "if {} != 0 goto {}", var, to),
            Instruction::Nop => write!(f, "nop"),
            Instruction::Print { var } => write!(f, "print {}", var),
            Instruction::State => write!(f, "state"),
        }
    }
}

// =================================================================================================
// Macros
// =================================================================================================

pub struct Macro {
    pub pattern: Regex,
    pub replacements: HashMap<String, usize>,
    pub instructions: Vec<String>,
}

impl Macro {
    pub fn parse(def: &str) -> Self {
        let escape_regex: Regex = Regex::new(r"[+*.$^()|?\\\[\]]").unwrap();
        let def = escape_regex.replace_all(def, |caps: &Captures| format!(r"\{}", &caps[0]));

        let macro_def_regex: Regex = Regex::new(r"\{(\w+)}").unwrap();
        let pattern = macro_def_regex.replace_all(&def, r"(\w+)");
        let pattern = Regex::new(&format!("^{}$", pattern)).unwrap();

        let mut replacements = HashMap::new();
        for (n, caps) in macro_def_regex.captures_iter(&def).flatten().enumerate() {
            replacements.insert(caps[1].to_string(), n);
        }

        Macro { pattern, replacements, instructions: Vec::new() }
    }
}

// =================================================================================================
// Parser
// =================================================================================================

pub struct Program {
    pub instructions: Vec<Instruction>,
    pub labels: HashMap<Label, usize>,
    pub macros: Vec<Macro>,
    max_temp_var: usize,
    max_labels: [usize; 5],
}

impl Program {
    pub fn from_file(file: &File) -> Result<Self, Box<dyn Error>> {
        let mut program = Program {
            instructions: Vec::new(),
            labels: HashMap::new(),
            macros: Vec::new(),
            max_temp_var: 0,
            max_labels: [0; 5],
        };
        let mut current_macro: Option<Box<Macro>> = None;

        // Read source file and append its lines to prologue
        let reader = BufReader::new(file);
        let lines: Vec<_> = PROLOGUE.lines()
            .map(|str| str.to_string())
            .chain(reader.lines().map_while(Result::ok))
            .enumerate()
            .collect();

        // Variable and label counting pre-pass
        let var_regex: Regex = Regex::new(r"\bz(\d+)\b").unwrap();
        let label_regex: Regex = Regex::new(r"([A-E])(\d+)").unwrap();
        for (_, line) in &lines {
            program.max_temp_var = var_regex.captures_iter(line).flatten()
                .map(|caps| caps[1].parse::<usize>().unwrap())
                .fold(program.max_temp_var, usize::max);

            program.max_labels = label_regex.captures_iter(line).flatten()
                .map(|caps| parse_label_capture(&caps))
                .fold(program.max_labels, |labels, (group, number)| {
                    if number > labels[group] {
                        let mut new_labels = labels;
                        new_labels[group] = number;
                        new_labels
                    } else {
                        labels
                    }
                })
        }

        for (line_num, line) in lines {
            if line.is_empty() || line.starts_with('#') {
                continue; // Skip empty lines and comments
            }

            if line.starts_with('@') {
                // Process directives:
                if let Some(line) = line.strip_prefix("@def") {
                    if current_macro.is_some() {
                        return Err(ParseError::boxed("Unexpected nested @def directive", line_num));
                    } else {
                        current_macro = Some(Box::new(Macro::parse(line.trim())));
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
                    &mut self.max_labels,
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
        labels: &mut HashMap<Label, usize>,
        line_num: usize,
    ) -> Result<&'a str, Box<dyn Error>> {
        let label_regex: Regex = Regex::new(r"^\[(\w+)]").unwrap();
        match label_regex.captures(instruction)? {
            Some(caps) => {
                let full = &caps[0];
                let label = Label::parse(&caps[1], line_num)?;

                if labels.contains_key(&label) {
                    return Err(ParseError::boxed(&format!("Redefined label {}", label), line_num));
                }

                labels.insert(label, instruction_number);
                Ok(instruction.strip_prefix(full).unwrap())
            }
            None => Ok(instruction)
        }
    }

    fn expand_macro(
        macros: &Vec<Macro>,
        m: &Macro,
        instructions: &mut Vec<Instruction>,
        labels: &mut HashMap<Label, usize>,
        max_temp_var: &mut usize,
        max_labels: &mut [usize; 5],
        caps: &Captures,
        line_num: usize,
    ) -> Result<(), Box<dyn Error>> {
        let auto_var_regex: Regex = Regex::new(r"\$(\w+)").unwrap();
        let auto_label_regex: Regex = Regex::new(r"%([A-E])(\d+)").unwrap();
        let mut auto_vars = HashMap::new();
        let mut auto_labels = HashMap::new();

        for instruction in &m.instructions {
            // Replace automatic labels
            let instruction = auto_label_regex.replace_all(instruction, |caps: &Captures| {
                let (group, number) = parse_label_capture(caps);
                let label = auto_labels.entry(Label::new(group, number)).or_insert_with(|| {
                    max_labels[group] += 1;
                    Label::new(group, max_labels[group])
                });

                format!("{}", label)
            });

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
                            max_labels,
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

fn parse_label_capture(caps: &Captures) -> (usize, usize) {
    (
        caps[1].chars().next().unwrap() as usize - 'A' as usize,
        caps[2].parse::<usize>().unwrap()
    )
}
