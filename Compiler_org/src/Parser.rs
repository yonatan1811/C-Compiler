use std::collections::btree_map::VacantEntry;
use std::fmt::format;
use std::panic;
use std::env;
use std::process;
use std::result;
use std::thread::panicking;
use crate::Lexer::Token;
use crate::Lexer::Lexer;

//this will be the output by the end
#[derive(Debug)]
pub enum ASTNode {
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
pub enum UnaryOp {
    Not,    // "!"
    BitNot, // "~"
    Negate, // "-"
}



#[derive(Debug)]
pub enum BinaryOp{
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



pub struct Parser {
    lexer: Lexer,
    current_token: Token,
}

impl Parser
{
    pub fn new(lexer: Lexer) -> Self {
        let mut parser = Self { current_token : Token::EOF, lexer };
        parser.current_token = parser.lexer.next_token();
        parser
    }

    pub fn eat(&mut self, expected: Token) {
        if self.current_token == expected {
            self.current_token = self.lexer.next_token();
        } else {
            panic!("Unexpected token: {:?}, expected: {:?}", self.current_token, expected);
        }
    }


    //changing parse to promgram parsing. cause we had parsing function as main, later on we will have functions so that will mess this up.
    pub fn parse(&mut self) -> ASTNode {
        let mut functions = Vec::new();
        while self.current_token != Token::EOF {
            functions.push(self.parse_function());
        }
        ASTNode::Program(functions)
    }

    // Function parsing: `int main() { return 100; }`
    fn parse_function(&mut self) -> ASTNode {
        let return_type = self.parse_type();
        let name = if let Token::Ident(name) = self.current_token.clone() {
            self.eat(Token::Ident(name.clone()));
            name
        } else {
            panic!("Expected function name");
        };

        self.eat(Token::LParen);
        self.eat(Token::RParen);
        self.eat(Token::LBrace);

        let mut body = Vec::new();
        while self.current_token != Token::RBrace {
            body.push(self.parse_statement());
        }


        self.eat(Token::RBrace);

        ASTNode::Function { name, return_type ,body }
    }


    fn parse_type(&mut self) ->String{
        match self.current_token.clone() {
            Token::Keyword(keyword) if keyword == "int" =>{
                self.eat(Token::Keyword("int".to_string()));
                return "int".to_string();
            }
            Token::Keyword(keyword) if keyword== "void" =>{
                self.eat(Token::Keyword("void".to_string()));
                return "void".to_string();
            }
            _ => panic!("Expected type keyword , supporting int and void"),
        }
    }





    fn parse_statement(&mut self) -> ASTNode {
        match self.current_token.clone() {
            Token::Keyword(keyword) if keyword == "return" => self.parse_return(),
            Token::Keyword(keyword) if keyword == "int" => self.parse_Assign_Or_declare(), // Handle declaration
            Token::Ident(_) => self.parse_assignment_or_expression(), // Handle variable assignment
            _ => panic!("Unexpected statement"),
        }
    }

    
    
    
    //All the parse expressions go here : 
    
    fn parse_expression(&mut self) -> Option<ASTNode> {
        // Check if it's a variable declaration (e.g., int a;)
        if let Token::Keyword(keyword) = self.current_token.clone() {
            if keyword == "int" {
                return Some(self.parse_Assign_Or_declare());
            }
        }
    
        // Start with logical OR expressions
        let left = self.parse_logical_or_expression();
        // Check if it's an assignment (e.g., a = 2;)
        if let ASTNode::Var(var_name) = &left {
            if self.current_token == Token::Assign {
                self.eat(Token::Assign);
                let right = self.parse_expression().unwrap();
                return Some(ASTNode::Assign(var_name.clone(), Box::new(right)));
            }
        }
    
        Some(left) // Return whatever expression was parsed
    }
    
    
    
    fn parse_assignment_or_expression(&mut self) -> ASTNode {
        let var_name = if let Token::Ident(name) = self.current_token.clone() {
            self.eat(Token::Ident(name.clone()));
            name
        } else {
            panic!("Expected an identifier");
        };
        
        if self.current_token == Token::Assign {
            self.eat(Token::Assign);
            let expr = self.parse_expression().unwrap();
            self.eat(Token::Semi);
            return ASTNode::Assign(var_name, Box::new(expr));
        }
    
        panic!("Unexpected token after identifier: {:?}", self.current_token);
    }


    fn parse_Assign_Or_declare(&mut self) -> ASTNode{
        self.eat(Token::Keyword("int".to_string()));
        //we are expecting an identifier , a name , so we will handle it accordingly
        let var_name = if let Token::Ident(name) = self.current_token.clone() {
            self.eat(Token::Ident(name.clone()));
            name
        } else {
            panic!("Expected an identifier after 'int'");
        };

        let mut init_expr = None;
    
        // Check for optional assignment
        if self.current_token == Token::Assign {
            self.eat(Token::Assign); // Consume '='
            init_expr = Some(Box::new(self.parse_expression().unwrap())); // Parse the expression
        }
        //at the end of the expressio we are expecting a semi colomn ;
        self.eat(Token::Semi);

        if let Some(expr) = init_expr {
            ASTNode::Assign(var_name, expr) 
        } else {
            ASTNode::Declare(var_name, None) 
        }
    }

    
    
    
    
    
    
    

    fn parse_logical_or_expression(&mut self) -> ASTNode{
        let mut left = self.parse_logical_and_expression();
        while let Token::LogOr = self.current_token{
            let op = match self.current_token{
                Token::LogOr => BinaryOp::LogOr,
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
        while let Token::LogAnd = self.current_token{
            let op = match self.current_token{
                Token::LogAnd => BinaryOp::LogAnd,
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
        while let Token::NEqualTo | Token::EqualTo = self.current_token{
            let op = match self.current_token{
                Token::NEqualTo => BinaryOp::NotEq,
                Token::EqualTo => BinaryOp::Equal,
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
        while let Token::Less | Token::GreatTh | Token::LessEq | Token::GreatThEq = self.current_token{
            let op = match self.current_token{
                Token::Less => BinaryOp::Less,
                Token::GreatTh => BinaryOp::Greater,
                Token::LessEq => BinaryOp::LessEq,
                Token::GreatThEq => BinaryOp::GreaterEq,
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
        while let Token::Plus | Token::Minus = self.current_token {
            let op = match self.current_token {
                Token::Plus => BinaryOp::Addition,
                Token::Minus => BinaryOp::Subtraction, // Now we are sure this is subtraction, not negation
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
    
        while let Token::Star | Token::Slash = self.current_token {
            let op = match self.current_token {
                Token::Star => BinaryOp::Multiplication,
                Token::Slash => BinaryOp::Division,
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
            Token::Number(value) => {
                self.eat(Token::Number(value));
                ASTNode::Constant(value as i64)
            }
            Token::Minus | Token::bitwise | Token::logical => {
                let op = match self.current_token {
                    Token::Minus => UnaryOp::Negate,
                    Token::bitwise => UnaryOp::BitNot,
                    Token::logical => UnaryOp::Not,
                    _ => unreachable!(),
                };
                self.eat(self.current_token.clone());
                let expr = self.parse_factor(); // Recursively parse next factor
                ASTNode::UnaryOp(op, Box::new(expr))
            }
            Token::LParen => {
                self.eat(Token::LParen);
                let expr = self.parse_expression().unwrap(); // Handle parentheses
                self.eat(Token::RParen);
                expr
            }
            Token::Ident(var_name) => { // Handle return of variables.. like return a ...
                let name = var_name.clone();
                self.eat(Token::Ident(name.clone()));
                ASTNode::Var(name)
            }
            _ =>{
                panic!("Unexpected token in factor")
            } 
        }
    }
    
    fn parse_return(&mut self) -> ASTNode {
        self.eat(Token::Keyword("return".to_string()));
        let value = match self.parse_expression() {
            Some(expr) => expr, // Valid expression
            None => ASTNode::Constant(0), // Handle empty return
        };
    
        self.eat(Token::Semi);
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



pub fn prettyPrinting(ast : &ASTNode){
    println!("{:?}" , ast);
}
