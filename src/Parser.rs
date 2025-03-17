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
    Function {
        name: String,
        body: Vec<ASTNode>,
        return_type: String,
    },
    Block(Vec<ASTNode>),
    UnaryOp(UnaryOp , Box<ASTNode>),
    BinaryOp(Box<ASTNode> , BinaryOp , Box<ASTNode>),
    Return(f64),
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


    fn parse(&mut self) -> ASTNode{
        self.parse_function()
    }

    // Function parsing: `int main() { return 100; }`
    fn parse_function(&mut self) -> ASTNode {
        let return_type = self.parse_type();
        let name = if let Lexer::Token::Ident(name) = self.current_token.clone() {
            self.eat(Lexer::Token::Ident(name.clone()));
            name
        } else {
            panic!("Expected function name");
        };
        
        
        self.eat(Lexer::Token::LParen);
        self.eat(Lexer::Token::RParen);
        self.eat(Lexer::Token::LBrace);

        //say we don't have arguments just yet

        let mut body = Vec::new();
        while self.current_token != Lexer::Token::RBrace {
            body.push(self.parse_statement());
        }

        self.eat(Lexer::Token::RBrace);

        ASTNode::Function {
            name,
            return_type,
            body,
        }
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
            Lexer::Token::LBrace => self.parse_block(),
            _ => panic!("Unexpected statement"),
        }
    }

    fn parse_block(&mut self) ->ASTNode{
        self.eat(Lexer::Token::LBrace);
        let mut statements = Vec::new();
        while self.current_token != Lexer::Token::RBrace {
            statements.push(self.parse_statement());
        }
        self.eat(Lexer::Token::RBrace);
        ASTNode::Block(statements)
    }


    fn parse_expression(&mut self) -> ASTNode{
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
                ASTNode::Return(value) // Return number as a node
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
                let expr = self.parse_expression(); // Handle parentheses
                self.eat(Lexer::Token::RParen);
                expr
            }
            _ => panic!("Unexpected token in factor"),
        }
    }
    
    fn parse_return(&mut self) -> ASTNode{
        self.eat(Lexer::Token::Keyword("return".to_string()));
        let value = self.parse_expression();
        self.eat(Lexer::Token::Semi);
        value
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

    pub fn get_return_type(&self) -> Option<&String> {
        if let ASTNode::Function { return_type, .. } = self {
            Some(return_type)
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

    pub fn get_return_value(&self) -> Option<f64> {
        match self {
            ASTNode::Return(value) => Some(*value),
            ASTNode::Function { body, .. } => {
                for stmt in body {
                    if let Some(value) = stmt.get_return_value() {
                        return Some(value);
                    }
                }
                None
            }
            ASTNode::Block(statements) => {
                for stmt in statements {
                    if let Some(value) = stmt.get_return_value() {
                        return Some(value);
                    }
                }
                None
            }
            _ => None,
        }
    }

    pub fn get_block_statements(&self) -> Option<&Vec<ASTNode>> {
        if let ASTNode::Block(statements) = self {
            Some(statements)
        } else {
            None
        }
    }
}



pub fn genASm(ast: &ASTNode) -> String {
    match ast {
        ASTNode::Function { name, body, .. } => {
            let mut result = format!(".global {}\n{}:\n", name, name);
            for stmt in body {
                result.push_str(&genASm(stmt));
            }
            result.push_str("ret\n");  // Ensure `ret` is properly formatted
            result
        }
        ASTNode::UnaryOp(op, expr) => {
            genUnary(op, expr)
            
        }, // Apply unary operation before returning
        ASTNode::BinaryOp(expr ,op, expr2) =>{
            genBinary(expr , op ,expr2)
        }
        ASTNode::Block(statements) => {
            let mut result = String::new();
            for stmt in statements {
                result.push_str(&genASm(stmt));
                println!("{:?}" ,result);
            }
            result
        }
        ASTNode::Return(value) => format!(""), // Return value
    }
}

pub fn genFunc(ast : &ASTNode) -> String
{
    let mut name = ast.get_name().unwrap();
    let mut body = ast.get_body();
    let mut res = genRet(ast.get_return_value().unwrap());
    format!(".global {name}\n{name}:\n{res}")
}

pub fn genBinary(expr : &ASTNode , op : &BinaryOp  , expr2 : &ASTNode) -> String{
    let mut res = genASm(expr);
    let mut res2 = genASm(expr2);

    if let ASTNode::Return(value) = expr {
        res = genMov(*value);  
    }
    
    if let ASTNode::Return(value) = expr2 {
        res2 = genMov(*value);  
    }

    match op {
        BinaryOp::Addition => format!(
            "{}\npush %rax\n{}\npop %rcx\naddq %rcx, %rax\n", 
            res, res2
        ),
    
        BinaryOp::Subtraction => format!(
            "{}\npush %rax\n{}\npop %rcx\nsubq %rcx, %rax\n", 
            res2, res // Order matters: `res2 - res`
        ),
    
        BinaryOp::Multiplication => format!(
            "{}\npush %rax\n{}\npop %rcx\nimulq %rcx, %rax\n", 
            res, res2
        ),
    
        BinaryOp::Division => format!(
            "{}\npush %rax         # Save e1 on stack\n{}\npop %rcx          # Load e1 into rcx\nmovq %rcx, %rax   # Move e1 into rax\ncqo               # Sign-extend rax into rdx:rax\nidivq %rcx        # rax = e1 / e2 (quotient), rdx = remainder\n", 
            res, res2
        ),
    
        _=> {
            println!("{:?}" , op);
            panic!("Unknown binary operation.")
        }
    }
}

pub fn genRet(value : f64) -> String{
    format!("ret")
}

pub fn genMov(value : f64) ->String{
    format!("movl ${} , %eax" , value as i32)
}



pub fn genUnary(op: &UnaryOp, expr: &ASTNode) -> String {
    // Recursively generate assembly for the inner expression
    let mut res = genASm(expr);  


    if let ASTNode::Return(value) = expr {
        res = genMov(*value);  
    }

    // Apply the correct unary operation
    match op {
        UnaryOp::Negate => format!("{res}\nneg %eax\n"),  // Negate %eax
        UnaryOp::BitNot => format!("{res}\nnot %eax\n"),  // Bitwise NOT
        UnaryOp::Not => format!(
            "{res}\ncmp $0, %eax\nsete %al\nmovzbl %al, %eax\n"  // Logical NOT
        ),
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
