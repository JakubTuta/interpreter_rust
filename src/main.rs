mod lexer;
use lexer::Lexer;

fn main() {
    let examples = ["1 + 2", "2 * 3", "3 / 1", "6", "asd", "123asd", "123.45asd"];

    for example in examples {
        println!("{example}");
        match Lexer::new().tokenize(example) {
            Ok(tokens) => println!("{:?}", tokens),
            Err(e) => println!("Error: {e}"),
        }
    }
}
