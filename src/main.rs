extern crate compiler_core;
use compiler_core::lexer::*;
fn main() {
    // let x = 2.0e4;
    let mut lexer = Lexer::new(".2");
    loop{
        match lexer.next_token() {
            Ok(TokenType::EOF) => break,
            Ok(token) => println!("{:#?}",token),
            Err(err)=>println!("{:#?}",err)
        };
    } 
}
