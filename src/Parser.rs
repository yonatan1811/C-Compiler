use std::collections::btree_map::VacantEntry;
use std::fmt::format;
//use std::intrinsics::unreachable;
use std::panic;
use std::env;
use std::fs::File;
use std::io::{self, Read}; // Fix for missing imports
use std::process;
use std::result;
use std::thread::panicking;
mod Lexer;

//this will be the output by the end
#[derive(Debug)]
enum ASTNode {
    //program
    Program(Vec<ASTNode>),  // Holds a list of functions

    //function declaration
    Function {
        name: String,
        body: Vec<ASTNode>,
        return_type : String,
    },

    //statement
    Return(Box<ASTNode>),  // Holds an expression

    Declare(String, Option<Box<ASTNode>>),  // Variable declaration (with optional initializer)

    Assign(String, Box<ASTNode>),  // Variable assignment

    //expr:
    Exp(Box<ASTNode>),  // Wraps expressions as statements

    Var(String),  // Represents variable access

    BinaryOp(Box<ASTNode>, BinaryOp, Box<ASTNode>),  // Binary operation

    UnaryOp(UnaryOp, Box<ASTNode>),  // Unary operation

    Constant(i64),  // Integer constants
}



#[derive(Debug)]
enum UnaryOp {
    Not,    // "!"
    BitNot, // "~"
    Negate, // "-"
}



#[derive(Debug)]
enum BinaryOp{
    Addition,
    Subtraction,
    Multiplication,
    Division,
    Less,
    Greater,
    LessEq,
    GreaterEq,
    NotEq,
    Equal,
    LogAnd,
    LogOr,
}



struct Parser {
    lexer: Lexer::Lexer,
    current_token: Lexer::Token,
}

impl Parser
{
    fn new(lexer: Lexer::Lexer) -> Self {
        let mut parser = Self { current_token : Lexer::Token::EOF, lexer };
        parser.current_token = parser.lexer.next_token();
        parser
    }

    fn eat(&mut self, expected: Lexer::Token) {
        if self.current_token == expected {
            self.current_token = self.lexer.next_token();
        } else {
            panic!("Unexpected token: {:?}, expected: {:?}", self.current_token, expected);
        }
    }


    //changing parse to promgram parsing. cause we had parsing function as main, later on we will have functions so that will mess this up.
    fn parse(&mut self) -> ASTNode {
        let mut functions = Vec::new();
        while self.current_token != Lexer::Token::EOF {
            functions.push(self.parse_function());
        }
        ASTNode::Program(functions)
    }

    // Function parsing: `int main() { return 100; }`
    fn parse_function(&mut self) -> ASTNode {
        let return_type = self.parse_type();
        let name = if let Lexer::Token::Ident(name) = self.current_token.clone() {
            self.eat(Lexer::Token::Ident(name.clone()));
            name
        } else {
            println!("{:?}" , self.current_token);
            panic!("Expected function name");
        };

        self.eat(Lexer::Token::LParen);
        self.eat(Lexer::Token::RParen);
        self.eat(Lexer::Token::LBrace);

        let mut body = Vec::new();
        while self.current_token != Lexer::Token::RBrace {
            body.push(self.parse_statement());
        }

        self.eat(Lexer::Token::RBrace);

        ASTNode::Function { name, return_type ,body }
    }


    fn parse_type(&mut self) ->String{
        match self.current_token.clone() {
            Lexer::Token::Keyword(keyword) if keyword == "int" =>{
                self.eat(Lexer::Token::Keyword("int".to_string()));
                return "int".to_string();
            }
            Lexer::Token::Keyword(keyword) if keyword== "void" =>{
                self.eat(Lexer::Token::Keyword("void".to_string()));
                return "void".to_string();
            }
            _ => panic!("Expected type keyword , supporting int and void"),
        }
    }





    fn parse_statement(&mut self) -> ASTNode {
        match self.current_token.clone() {
            Lexer::Token::Keyword(keyword) if keyword == "return" => self.parse_return(),
            Lexer::Token::Keyword(keyword) if keyword == "int" => self.parse_expression().unwrap(),

            _ => panic!("Unexpected statement"),
        }
    }

    
    //Declaretion stuff :

    //This function will handle declarations like : int x = 3; or int b = 2; int x = 3*(b*2); or int x;

    fn parse_Assign_Or_declare(&mut self) -> ASTNode{
        self.eat(Lexer::Token::Keyword("int".to_string()));
        //we are expecting an identifier , a name , so we will handle it accordingly
        let var_name = if let Lexer::Token::Ident(name) = self.current_token.clone() {
            self.eat(Lexer::Token::Ident(name.clone()));
            name
        } else {
            panic!("Expected an identifier after 'int'");
        };

        let mut init_expr = None;
    
        // Check for optional assignment
        if self.current_token == Lexer::Token::Assign {
            self.eat(Lexer::Token::Assign); // Consume '='
            init_expr = Some(Box::new(self.parse_expression().unwrap())); // Parse the expression
        }
        //at the end of the expressio we are expecting a semi colomn ;
        self.eat(Lexer::Token::Semi);
        match init_expr {
            Some(expr) => ASTNode::Assign(var_name, expr),
            None => ASTNode::Declare(var_name, init_expr),
        }
    }

    
    //All the parse expressions go here : 
    
    fn parse_expression(&mut self) -> Option<ASTNode> {
        let left = self.parse_logical_or_expression(); // Start with the lowest precedence expression
    
        // Check if the left-hand side is an identifier for assignment
        if let (ASTNode::Var(var_name)) = left {
            if let Lexer::Token::Assign = self.current_token {
                self.eat(self.current_token.clone());
    
                let right = self.parse_expression()?; // Parse right-hand side expression
                return Some(ASTNode::Assign(var_name, Box::new(right))); // Now `var_name` is a String
            }
    
            return Some(ASTNode::Var(var_name)); // If no assignment, just return the variable
        }
    
        Some(left) // Return whatever expression was parsed
    }
    
    
    
    
    

    fn parse_logical_or_expression(&mut self) -> ASTNode{
        let mut left = self.parse_logical_and_expression();
        while let Lexer::Token::LogOr = self.current_token{
            let op = match self.current_token{
                Lexer::Token::LogOr => BinaryOp::LogOr,
                _ => unreachable!(),
            };
            self.eat(self.current_token.clone());
            let right = self.parse_logical_and_expression();
            left = ASTNode::BinaryOp(Box::new(left) , op , Box::new(right));
        }
        left
    }
    


    fn parse_logical_and_expression(&mut self) ->ASTNode{
        let mut left = self.parse_equality_expression();
        while let Lexer::Token::LogAnd = self.current_token{
            let op = match self.current_token{
                Lexer::Token::LogAnd => BinaryOp::LogAnd,
                _ => unreachable!(),
            };
            self.eat(self.current_token.clone());
            let right = self.parse_equality_expression();
            left = ASTNode::BinaryOp(Box::new(left) , op , Box::new(right));
        }
        left
    }

    fn parse_equality_expression(&mut self) ->ASTNode{
        let mut left = self.parse_relational_expression();
        while let Lexer::Token::NEqualTo | Lexer::Token::EqualTo = self.current_token{
            let op = match self.current_token{
                Lexer::Token::NEqualTo => BinaryOp::NotEq,
                Lexer::Token::EqualTo => BinaryOp::Equal,
                _ => unreachable!(),
            };
            self.eat(self.current_token.clone());
            let right = self.parse_relational_expression();
            left = ASTNode::BinaryOp(Box::new(left) , op , Box::new(right));
        }
        left
    }

    fn parse_relational_expression(&mut self) -> ASTNode{

        let mut left = self.parse_add_expression();
        while let Lexer::Token::Less | Lexer::Token::GreatTh | Lexer::Token::LessEq | Lexer::Token::GreatThEq = self.current_token{
            let op = match self.current_token{
                Lexer::Token::Less => BinaryOp::Less,
                Lexer::Token::GreatTh => BinaryOp::Greater,
                Lexer::Token::LessEq => BinaryOp::LessEq,
                Lexer::Token::GreatThEq => BinaryOp::GreaterEq,
                _ => unreachable!(),
            };
            self.eat(self.current_token.clone());
            let right = self.parse_add_expression();
            left = ASTNode::BinaryOp(Box::new(left) , op , Box::new(right));
        }
        left
    }


    fn parse_add_expression(&mut self) ->ASTNode{
        let mut node = self.parse_term(); // Start with the term (multiplication first)
        while let Lexer::Token::Plus | Lexer::Token::Minus = self.current_token {
            let op = match self.current_token {
                Lexer::Token::Plus => BinaryOp::Addition,
                Lexer::Token::Minus => BinaryOp::Subtraction, // Now we are sure this is subtraction, not negation
                _ => unreachable!(),
            };
            self.eat(self.current_token.clone());
            let right = self.parse_term(); // Ensure proper precedence
            node = ASTNode::BinaryOp(Box::new(node), op , Box::new(right));
        }
        node
    }

    fn parse_term(&mut self) -> ASTNode {
        let mut left = self.parse_factor(); // Start with a factor
    
        while let Lexer::Token::Star | Lexer::Token::Slash = self.current_token {
            let op = match self.current_token {
                Lexer::Token::Star => BinaryOp::Multiplication,
                Lexer::Token::Slash => BinaryOp::Division,
                _ => unreachable!(),
            };
            self.eat(self.current_token.clone()); // Consume the operator
            let right = self.parse_factor(); // Parse the next factor
    
            left = ASTNode::BinaryOp(Box::new(left), op, Box::new(right));
        }
        left
    }

    fn parse_factor(&mut self) -> ASTNode {
        match self.current_token.clone() {
            Lexer::Token::Number(value) => {
                self.eat(Lexer::Token::Number(value));
                ASTNode::Constant(value as i64)
            }
            Lexer::Token::Minus | Lexer::Token::bitwise | Lexer::Token::logical => {
                let op = match self.current_token {
                    Lexer::Token::Minus => UnaryOp::Negate,
                    Lexer::Token::bitwise => UnaryOp::BitNot,
                    Lexer::Token::logical => UnaryOp::Not,
                    _ => unreachable!(),
                };
                self.eat(self.current_token.clone());
                let expr = self.parse_factor(); // Recursively parse next factor
                ASTNode::UnaryOp(op, Box::new(expr))
            }
            Lexer::Token::LParen => {
                self.eat(Lexer::Token::LParen);
                let expr = self.parse_expression().unwrap(); // Handle parentheses
                self.eat(Lexer::Token::RParen);
                expr
            }
            _ =>{
                panic!("Unexpected token in factor")

            } 
        }
    }
    
    fn parse_return(&mut self) -> ASTNode {
        self.eat(Lexer::Token::Keyword("return".to_string()));
    
        let value = match self.parse_expression() {
            Some(expr) => expr, // Valid expression
            None => ASTNode::Constant(0), // Handle empty return
        };
    
        self.eat(Lexer::Token::Semi);
        ASTNode::Return(Box::new(value))
    }

}


impl ASTNode {
    pub fn get_name(&self) -> Option<&String> {
        if let ASTNode::Function { name, .. } = self {
            Some(name)
        } else {
            None
        }
    }
    pub fn get_body(&self) -> Option<&Vec<ASTNode>> {
        if let ASTNode::Function { body, .. } = self {
            Some(body)
        } else {
            None
        }
    }

}

pub fn genASm(ast: &ASTNode) -> String {
    match ast {
        ASTNode::Program(functions) => {
            let mut result = String::new();
            for func in functions {
                result.push_str(&genASm(func));
            }
            result
        }
        ASTNode::Function { name, return_type, body } => {
            let mut result = format!(".global {}\n{}:\n", name, name);
            for stmt in body {
                result.push_str(&genASm(stmt));
            }
            result
        }
        ASTNode::Return(value) => format!("{}ret\n", genASm(value)),
        ASTNode::Constant(val) => format!("movq ${} , %rax\n", val),
        ASTNode::Var(name) => format!("movq $[{}] , %rax\n", name),
        ASTNode::BinaryOp(left, op, right) => {
            let res1 = genASm(left);
            let res2 = genASm(right);

            match op {
                BinaryOp::LogAnd => {
                    let label_false = new_label("false");
                    let label_end = new_label("end");

                    return format!(
                        "{}\n\
                        testq %rax, %rax\n\
                        jz {}\n\
                        {}\n\
                        testq %rax, %rax\n\
                        jz {}\n\
                        movq $1, %rax\n\
                        jmp {}\n\
                        {}:\n\
                        movq $0, %rax\n\
                        {}:\n",
                        res1, label_false, res2, label_false, label_end, label_false, label_end
                    );
                }
                BinaryOp::LogOr => {
                    let label_true = new_label("true");
                    let label_end = new_label("end");

                    return format!(
                        "{}\n\
                        testq %rax, %rax\n\
                        jnz {}\n\
                        {}\n\
                        testq %rax, %rax\n\
                        jnz {}\n\
                        movq $0, %rax\n\
                        jmp {}\n\
                        {}:\n\
                        movq $1, %rax\n\
                        {}:\n",
                        res1, label_true, res2, label_true, label_end, label_true, label_end
                    );
                }
                _ => {}
            }

            let op_asm = match op {
                BinaryOp::Addition => "addq %rcx, %rax\n",
                BinaryOp::Subtraction => "subq %rcx, %rax\n",
                BinaryOp::Multiplication => "imulq %rcx, %rax\n",
                BinaryOp::Division => {
                    "movq $0 , %rdx \nidiv %rcx\n"
                }
                BinaryOp::Equal => {
                    "cmpq %rcx, %rax\nsete %al\nmovzbq %al, %rax\n"
                }
                BinaryOp::NotEq => {
                    "cmpq %rcx, %rax\nsetne %al\nmovzbq %al, %rax\n"
                }
                BinaryOp::Less => {
                    "cmpq %rcx, %rax\nsetl %al\nmovzbq %al, %rax\n"
                }
                BinaryOp::LessEq => {
                    "cmpq %rcx, %rax\nsetle %al\nmovzbq %al, %rax\n"
                }
                BinaryOp::Greater => {
                    "cmpq %rcx, %rax\nsetg %al\nmovzbq %al, %rax\n"
                }
                BinaryOp::GreaterEq => {
                    "cmpq %rcx, %rax\nsetge %al\nmovzbq %al, %rax\n"
                }
                BinaryOp::LogAnd => unreachable!(),
                BinaryOp::LogOr => unreachable!(),
            };

            format!("{}\npush %rax\n{}\npop %rcx\n{}", res1, res2, op_asm)
        }
        ASTNode::UnaryOp(op, expr) => {
            let res = genASm(expr);
            let op_asm = match op {
                UnaryOp::Negate => "neg %rax\n",
                UnaryOp::BitNot => "not %rax\n",
                UnaryOp::Not => "cmp $0, %rax\nsete %al\nmovzb %al, %rax\n",
            };
            format!("{}{}", res, op_asm)
        }
        _ => String::new(),
    }
}


static mut LABEL_COUNT: usize = 0;
fn new_label(base: &str) -> String {
    unsafe {
        LABEL_COUNT += 1;
        format!(".{}_{}", base, LABEL_COUNT)
    }
}

pub fn prettyPrinting(ast : &ASTNode){
    println!("{:?}" , ast);
}


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
    let mut parser = Parser::new(lexer_tokens);

    let mut token = parser.current_token.clone();
    

    let ast = parser.parse();
    //prettyPrinting(&ast);
    println!("{}" , genASm(&ast));
    Ok(())
}
