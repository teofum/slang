use crate::machine::{Machine, State};
use crate::program::{Program, Variable};
use std::env;
use std::error::Error;
use std::fs::File;

mod program;
mod machine;
mod error;
mod prologue;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1).peekable();
    
    let mut print_code = false;
    if args.peek().is_some_and(|arg| arg == "-p") {
        args.next();
        print_code = true;
    }
    
    let program_file = File::open(args.next().unwrap())?;
    match Program::from_file(&program_file) {
        Ok(program) => {
            if print_code {
                println!("Program number: {}", program);
            } else {
                let mut machine = Machine::new(
                    State::from_vars(args.map(|arg| arg.parse::<usize>().unwrap()).collect()),
                    &program,
                );

                machine.run();

                println!("Y = {}", machine.state().get_var(&Variable::Y));
            }
        }
        Err(e) => {
            println!("\x1b[31;1m{}\x1b[0m", e);
        }
    };

    Ok(())
}
