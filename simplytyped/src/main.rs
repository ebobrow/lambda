use parser::Parser;
use scanner::Scanner;

use crate::typer::Typer;

mod parser;
mod scanner;
mod typer;

fn main() {
    let mut rl = rustyline::DefaultEditor::new().unwrap();
    loop {
        let readline = rl.readline("λ ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(&line).unwrap();
                if let Err(e) = type_line(line) {
                    eprintln!("{}", e);
                }
            }
            Err(_) => {
                println!("Connection terminated");
                break;
            }
        }
    }
}

fn type_line(line: String) -> anyhow::Result<()> {
    let tokens = Scanner::scan(line)?;
    let expr = Parser::parse(tokens)?;
    let ty = Typer::default().typecheck(&expr)?;
    println!("{}", ty.to_string());
    Ok(())
}
