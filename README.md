C Compiler in Rust

Welcome to my C compiler written in Rust! This project aims to create a fully functional C compiler from scratch, leveraging the power and safety of Rust. The compiler will handle parsing, semantic analysis, and code generation while supporting essential C features.

Features

Lexical Analysis: Tokenizes C source code into a list of tokens.

Parsing: Converts the token stream into an Abstract Syntax Tree (AST).

Semantic Analysis: Ensures type correctness and scope resolution.

Code Generation: Outputs machine code or intermediate representation (IR).

Error Handling: Provides clear and precise error messages.

Optimization: Future plans include optimization passes for better performance.

Progress
As for now i managed to create a lexer for a limited tokens.
A Parser that can handle c syntax. I can only handle arithmetic and logical equations for now. 


Usage

To build and run the compiler, ensure you have Rust installed and then execute:

Future Plans

Add support for more C features like structs, pointers, and loops.

Implement optimizations for better compiled code efficiency.

Introduce an interactive REPL mode for quick testing.


Stay tuned for updates as I continue developing this compiler!

