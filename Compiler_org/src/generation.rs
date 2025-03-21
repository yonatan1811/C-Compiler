use std::collections::HashMap;
use crate::Parser::BinaryOp;
use crate::Parser::UnaryOp;
use crate::Parser::ASTNode;

pub struct CodeGenContext {
    var_offsets: HashMap<String, i32>,
    stack_offset: i32,
}

impl CodeGenContext {
    pub fn new() -> Self {
        Self {
            var_offsets: HashMap::new(),
            stack_offset: 0, // Start at 0, grow downward (-8, -16, ...)
        }
    }

    pub fn allocate_var(&mut self, name: &str) -> i32 {
        self.stack_offset -= 8; // Each variable gets 8 bytes
        self.var_offsets.insert(name.to_string(), self.stack_offset);
        self.stack_offset
    }

    pub fn enter_function(&mut self) {
        self.stack_offset = 0; // Reset the stack offset for a new function
    }

    pub fn exit_function(&mut self) {
        self.stack_offset = 0; // Reset when exiting
    }

    pub fn get_var_offset(&self, name: &str) -> i32 {
        *self.var_offsets.get(name).unwrap_or_else(|| panic!("Variable {} not found", name))
    }

    pub fn get_var_exist(&self , name : &str) -> i32{
        if self.var_offsets.contains_key(name){
            1
        }
        else{
            0
        }
    }
}


pub fn genASm(ast: &ASTNode, context: &mut CodeGenContext) -> String {
    match ast {
        // Program containing multiple functions
        ASTNode::Program(functions) => {
            let mut result = String::new();
            for func in functions {
                result.push_str(&genASm(func, context));
            }
            result
        }

        // Function Declaration
        ASTNode::Function { name, body, return_type } => {
            let mut result = format!(
                ".global {}\n{}:\n\
                pushq %rbp\n\
                movq %rsp, %rbp\n",
                name, name
            );

            context.enter_function(); // Reset stack offset tracking

            for stmt in body {
                result.push_str(&genASm(stmt, context));
            }

            result.push_str("movq %rbp, %rsp\npopq %rbp\nret\n");
            context.exit_function();
            result
        }

        // Variable Declaration
        ASTNode::Declare(name, initializer) => {
            let offset = context.allocate_var(name);
            let mut result = format!("subq $8, %rsp  # Allocate space for {}\n", name);

            if let Some(expr) = initializer {
                let expr_code = genASm(expr, context);
                result.push_str(&format!(
                    "{}\nmovq %rax, {}(%rbp)  # Store value in {}\n",
                    expr_code, offset, name
                ));
            }

            result
        }

        // Variable Assignment
        ASTNode::Assign(name, expr) => {
            let expr_code = genASm(expr, context);
            let mut offset = 0;
            if context.get_var_exist(name) == 1{
                offset = context.get_var_offset(name);
            }
            else{
                offset = context.allocate_var(name);
            }
            format!(
                "{}\nmovq %rax, {}(%rbp)  # Assign value to {}\n",
                expr_code, offset, name
            )
        }

        // Return Statement
        ASTNode::Return(value) => format!("{}\n", genASm(value, context)),

        // Wrapping expressions
        ASTNode::Exp(expr) => genASm(expr, context),

        // Constants
        ASTNode::Constant(val) => format!("movq ${}, %rax\n", val),

        // Variable Usage
        ASTNode::Var(name) => {
            let offset = context.get_var_offset(name);
            format!("movq {}(%rbp), %rax  # Load variable {}\n", offset, name)
        }

        // Binary Operations
        ASTNode::BinaryOp(left, op, right) => {
            let res1 = genASm(left, context);
            let res2 = genASm(right, context);

            // Handle logical operators first (short-circuiting)
            if matches!(op, BinaryOp::LogAnd | BinaryOp::LogOr) {
                let label_short_circuit = new_label("short_circuit");
                let label_end = new_label("end");

                return match op {
                    BinaryOp::LogAnd => format!(
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
                        res1, label_short_circuit, res2, label_short_circuit, label_end,
                        label_short_circuit, label_end
                    ),
                    BinaryOp::LogOr => format!(
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
                        res1, label_short_circuit, res2, label_short_circuit, label_end,
                        label_short_circuit, label_end
                    ),
                    _ => unreachable!(), // Already checked matches!
                };
            }

            // Standard binary operations
            let op_asm = match op {
                BinaryOp::Addition => "addq %rcx, %rax\n",
                BinaryOp::Subtraction => "subq %rcx, %rax\n",
                BinaryOp::Multiplication => "imulq %rcx, %rax\n",
                BinaryOp::Division => "movq $0, %rdx\nidiv %rcx\n",
                BinaryOp::Equal => "cmpq %rcx, %rax\nsete %al\nmovzbq %al, %rax\n",
                BinaryOp::NotEq => "cmpq %rcx, %rax\nsetne %al\nmovzbq %al, %rax\n",
                BinaryOp::Less => "cmpq %rcx, %rax\nsetl %al\nmovzbq %al, %rax\n",
                BinaryOp::LessEq => "cmpq %rcx, %rax\nsetle %al\nmovzbq %al, %rax\n",
                BinaryOp::Greater => "cmpq %rcx, %rax\nsetg %al\nmovzbq %al, %rax\n",
                BinaryOp::GreaterEq => "cmpq %rcx, %rax\nsetge %al\nmovzbq %al, %rax\n",
                BinaryOp::LogAnd | BinaryOp::LogOr => unreachable!(), // Already handled above
            };

            format!(
                "{}\npushq %rax\n{}\npopq %rcx\n{}",
                res1, res2, op_asm
            )
        }

        // Unary Operations
        ASTNode::UnaryOp(op, expr) => {
            let res = genASm(expr, context);
            let op_asm = match op {
                UnaryOp::Negate => "neg %rax\n",
                UnaryOp::BitNot => "not %rax\n",
                UnaryOp::Not => "cmp $0, %rax\nsete %al\nmovzbq %al, %rax\n",
            };
            format!("{}{}", res, op_asm)
        }
    }
}


static mut LABEL_COUNT: usize = 0;
fn new_label(base: &str) -> String {
    unsafe {
        LABEL_COUNT += 1;
        format!(".{}_{}", base, LABEL_COUNT)
    }
}

