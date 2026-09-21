mod evaluator;
mod lexer;
mod parser;

use evaluator::Evaluator;
use lexer::Lexer;
use parser::Parser;

fn main() {
    let examples = [
        "1 + 2 * 3",
        "(1 + 2) * 3",
        "-5 + 3",
        "7 / 2",
        "1 + 2.5",
        "1 / 0",
        "(1 + 2",
    ];

    for example in examples {
        println!("{example}");
        let result = Lexer::new()
            .tokenize(example)
            .map_err(|e| e.to_string())
            .and_then(|tokens| Parser::new().parse(tokens).map_err(|e| e.to_string()))
            .and_then(|expr| Evaluator::new().evaluate(&expr).map_err(|e| e.to_string()));

        match result {
            Ok(value) => println!("  = {value:?}"),
            Err(e) => println!("  Error: {e}"),
        }
    }
}
