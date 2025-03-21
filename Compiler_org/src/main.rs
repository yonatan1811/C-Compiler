mod generation;
mod Parser;
mod Lexer;

use std::fs::File;
use std::io::{self, Read}; // Fix for missing imports
use std::fmt::format;
//use std::intrinsics::unreachable;
use std::panic;
use std::env;

fn main() -> io::Result<()>
{
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <filename>", args[0]);
        std::process::exit(1);
    }

    let file = &args[1];
    let mut contents = String::new();
    
    let mut openedfile = File::open(file)?; 

    openedfile.read_to_string(&mut contents)?; 
    let mut lexer_tokens = Lexer::Lexer::new(&contents);
    let mut parser = Parser::Parser::new(lexer_tokens);

    let ast = parser.parse();
    let mut context = generation::CodeGenContext::new();
    //prettyPrinting(&ast);
    println!("{}" , generation::genASm(&ast , &mut context));
    Ok(())
}