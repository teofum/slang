use crate::program::{Instruction, Program, Variable};

pub struct State {
    x: Vec<usize>,
    z: Vec<usize>,
    y: usize,
    pub pc: usize,
}

impl State {
    pub fn from_vars(vars: Vec<usize>) -> Self {
        State { x: vars, z: Vec::new(), y: 0, pc: 0 }
    }

    pub fn get_var(&self, var: &Variable) -> usize {
        match var {
            Variable::X(n) => if *n <= self.x.len() { self.x[*n - 1] } else { 0 },
            Variable::Z(n) => if *n <= self.z.len() { self.z[*n - 1] } else { 0 },
            Variable::Y => self.y,
        }
    }

    pub fn set_var(&mut self, var: &Variable, value: usize) {
        match var {
            Variable::X(n) => {
                while *n > self.x.len() { self.x.push(0); }
                self.x[*n - 1] = value;
            }
            Variable::Z(n) => {
                while *n > self.z.len() { self.z.push(0); }
                self.z[*n - 1] = value;
            }
            Variable::Y => self.y = value,
        }
    }
}

pub struct Machine<'a> {
    state: State,
    program: &'a Program,
}

impl<'a> Machine<'a> {
    pub fn new(initial_state: State, program: &'a Program) -> Self {
        Machine { state: initial_state, program }
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn step(&mut self) {
        if let Some(instruction) = self.program.instructions.get(self.state.pc) {
            let mut jumped = false;
            match instruction {
                Instruction::Increment { var } => self.state.set_var(var, self.state.get_var(var) + 1),
                Instruction::Decrement { var } => {
                    let val = self.state.get_var(var);
                    if val > 0 { self.state.set_var(var, val - 1); }
                }
                Instruction::JumpNonZero { var, to } => if self.state.get_var(var) > 0 {
                    // On jump to undefined label, halt execution
                    self.state.pc = *self.program.labels.get(to)
                        .unwrap_or(&self.program.instructions.len());
                    jumped = true;
                },
                Instruction::Nop => {}
            };

            if !jumped { self.state.pc += 1; }
        }
    }

    pub fn run(&mut self) {
        while self.state.pc < self.program.instructions.len() { self.step(); }
    }
}