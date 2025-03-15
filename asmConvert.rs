use std::panic::PanicInfo;



pub fn genASm(ast: &ASTNode) -> String {
    match ast {
        ASTNode::Function { name, body, .. } => {
            let mut result = format!(".global {}\n{}:\n", name, name);
            for stmt in body {
                result.push_str(&genASm(stmt));
            }
            result
        }
        ASTNode::UnaryOp(op, expr) => genUnary(op, expr),
        // ASTNode::Block(statements) => {
        //     let mut result = String::new();
        //     for stmt in statements {
        //         result.push_str(&genASm(stmt));
        //     }
        //     result
        // }
        ASTNode::Return(value) => genRet(*value),
    }
}

pub fn genFunc(ast : &ASTNode) -> String
{
    let mut name = ast.get_name().unwrap();
    let mut body = ast.get_body()
    let mut res = genRet(ast.get_return_value().unwrap());
    format!(".global {name}\n{name}:\n{res}")
}


pub fn genRet(value : f64) -> String{
    format!("movl ${value} , %eax\nret")
}


pub fn genUnary(op: &UnaryOp, expr: &ASTNode) -> String {
    let expr_code = genASm(expr);
    match op {
        UnaryOp::Negate => format!("{expr_code}\nneg %eax"),
        UnaryOp::BitNot => format!("{expr_code}\nnot %eax"),
        UnaryOp::Not => format!("{expr_code}\ncmp $0, %eax\nsete %al"),
    }
}