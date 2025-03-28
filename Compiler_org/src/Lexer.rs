use std::env;
use std::fs::File;
use std::io::{self, Read}; // Fix for missing imports
use std::process;
#[derive(Debug,Clone, PartialEq)]

pub enum Token {
    Number(f64),
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Semi,
    Ident(String),
    Keyword(String),
    Unknown,
    Func,
    EOF,

    negation,
    bitwise,
    logical,
    LogAnd,
    LogOr,
    
    EqualTo,
    NEqualTo,
    Less,
    LessEq,
    GreatTh,
    GreatThEq,

    Assign,

    Colon,
    Question,
}

pub struct Lexer {
    input: Vec<char>,
    pos: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            pos: 0,
        }
    }

    pub fn next_char(&mut self) -> Option<char> {
        if self.pos < self.input.len() {
            let ch = self.input[self.pos];
            self.pos += 1;
            Some(ch)
        } else {
            None
        }
    }

    fn peek_char(&self) -> Option<char> {
        if self.pos < self.input.len() {
            Some(self.input[self.pos])
        } else {
            None
        }
    }

    fn lex_number(&mut self, first_digit: char) -> Token {
        let mut num_str = first_digit.to_string();
        while let Some(ch) = self.peek_char() {
            if ch.is_numeric() || ch == '.' {
                num_str.push(self.next_char().unwrap());
            } else {
                break;
            }
        }
        
        // Improved error handling for number parsing
        match num_str.parse::<f64>() {
            Ok(num) => Token::Number(num),
            Err(_) => panic!("Failed to parse number: {}", num_str), // You can handle the error more gracefully
        }
    }


     // Lex an identifier (variable or function name)
     fn lex_identifier(&mut self, first_char: char) -> Token {
        let mut ident_str = first_char.to_string();
        while let Some(ch) = self.peek_char() {
            if ch.is_alphanumeric() || ch == '_' {
                ident_str.push(self.next_char().unwrap());
            } else if ch.is_numeric() {
                ident_str.push(self.next_char().unwrap());
            }
            else if ch == '&' || ch == '|'{
                ident_str.push(self.next_char().unwrap());
                break;
            }
            else
            {
                break;
            }
        }
        
        // Handle keywords
        match ident_str.as_str() {
            "int" | "return" | "if" | "else" => Token::Keyword(ident_str),
            "&&" => Token::LogAnd,
            "||" => Token::LogOr,
            _ => Token::Ident(ident_str),
        }
    }

    fn lex_identifier_then(&mut self , first_char : char) -> Token {
        let mut ident_str = first_char.to_string();
        let mut ch = self.peek_char();
        
        if ch == Some('=') {
            ident_str.push(self.next_char().unwrap());
        }
        
        match ident_str.as_str() {
            "==" => Token::EqualTo,
            "!=" => Token::NEqualTo,
            ">=" => Token::GreatThEq,
            "<=" => Token::LessEq,
            "=" => Token::Assign,
            "!" => Token::logical,
            ">" => Token::GreatTh,
            "<" => Token::Less,
            _ => panic!("Unknown"),
        }
    }


    pub fn next_token(&mut self) -> Token {
        while let Some(ch) = self.next_char() {
            return match ch {
                ' ' | '\t' | '\n' | '\r' => continue,
                '+' => Token::Plus,
                '-' => Token::Minus,
                '*' => Token::Star,
                '/' => Token::Slash,
                '(' => Token::LParen,
                ')' => Token::RParen,
                '{' => Token::LBrace,
                '}' => Token::RBrace,
                ';' => Token::Semi,
                ':' => Token::Colon,
                '?' => Token::Question,
                '~' => Token::bitwise,
                '!' | '=' | '<' | '>' => self.lex_identifier_then(ch),
                '0'..='9' => self.lex_number(ch),
                'a'..='z' | 'A'..='Z' | '_' | '&' | '|' => self.lex_identifier(ch), // Identifiers (including keywords)
                _ => panic!("Unexpected character: {}", ch),
            };
        }
        Token::EOF
    }
}