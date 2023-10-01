use parser::Parser;
use scanner::Scanner;

mod parser;
mod scanner;

fn main() {
    let mut rl = rustyline::DefaultEditor::new().unwrap();
    loop {
        let readline = rl.readline("λ ");
        match readline {
            Ok(line) => {
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
    println!("{:?}", expr);
    Ok(())
}
