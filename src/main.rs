mod lexer;
mod parser;

use lexer::Lexer;
use parser::Parser;

fn main() {
    let examples = ["1 + 2 * 3", "(1 + 2) * 3", "-5 + 3", "(1 + 2"];

    for example in examples {
        println!("{example}");
        match Lexer::new().tokenize(example) {
            Ok(tokens) => {
                println!("  {tokens:?}");
                match Parser::new().parse(tokens) {
                    Ok(expr) => println!("  {expr:?}"),
                    Err(e) => println!("  Parse error: {e}"),
                }
            }
            Err(e) => println!("  Lex error: {e}"),
        }
        println!();
    }
}
