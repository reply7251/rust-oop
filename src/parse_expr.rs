use syn::{Expr, Block, Pat, Stmt, parse2};
use quote::quote;

fn parse_statement(stmt: &mut Stmt) {

}

fn parse_pattern(pat: &mut Pat) {

}

fn parse_block(block: &mut Block) {
    for line in &mut block.stmts {
        parse_statement(line);
    }
}

fn parse_expr(expr: &mut Expr) {
    match expr {
        Expr::Array(x) => {
            for elem in &mut x.elems {
                parse_expr(elem);
            }
        },
        Expr::Assign(x) => { parse_expr(&mut x.right); },
        Expr::AssignOp(x) => { parse_expr(&mut x.right); },
        Expr::Async(x) => { 
            for stmt in &mut x.block.stmts {
                parse_statement(stmt);
            }
        },
        Expr::Await(x) => {
            parse_expr(&mut x.base);
        },
        Expr::Binary(x) => {
            parse_expr(&mut x.left);
            parse_expr(&mut x.right);
        },
        Expr::Block(x) => {
            for stmt in &mut x.block.stmts {
                parse_statement(stmt);
            }
        },
        Expr::Box(x) => {
            parse_expr(&mut x.expr);
        },
        Expr::Break(_) => todo!(),
        Expr::Call(x) => {
            for arg in &mut x.args {
                parse_expr(arg);
            }
            parse_expr(&mut x.func);
        },
        Expr::Cast(x) => {
            parse_expr(&mut x.expr);
        },
        Expr::Closure(x) => {
            parse_expr(&mut x.body);
        },
        Expr::Continue(_) => todo!(),
        Expr::Field(x) => {
            parse_expr(&mut x.base);
        },
        Expr::ForLoop(x) => {
            for line in &mut x.body.stmts {
                parse_statement(line);
            }
        },
        Expr::Group(x) => {
            parse_expr(&mut x.expr);
        },
        Expr::If(x) => {
            parse_expr(&mut x.cond);
            parse_block(&mut x.then_branch);
            if x.else_branch.is_some() {
                parse_expr(&mut x.else_branch.as_mut().unwrap().1);
            }
        },
        Expr::Index(x) => {
            parse_expr(&mut x.expr);
            
            parse_expr(&mut x.index);
        },
        Expr::Let(x) => {
            parse_expr(&mut x.expr);
        },
        Expr::Lit(_) => {},
        Expr::Loop(x) => {
            parse_block(&mut x.body);
        },
        Expr::Macro(x) => {
            let _expr: &mut Expr = &mut syn::parse2(x.mac.tokens.clone()).unwrap();
            parse_expr(_expr);
        },
        Expr::Match(x) => {
            parse_expr(&mut x.expr);
            for arm in &mut x.arms {
                parse_pattern(&mut arm.pat);
                parse_expr(&mut arm.body);
            }
        },
        Expr::MethodCall(method) => {
            parse_expr(&mut method.receiver);
            for arg in &mut method.args {
                parse_expr(arg);
            }
        },
        Expr::Paren(x) => {
            parse_expr(&mut x.expr);
        },
        Expr::Path(x) => {
            let segments = &mut x.path.segments;
            if segments.len() == 1 {
                if &segments[0].ident.to_string() == "self" {
                    segments.clear();
                    segments.push(parse2(quote!{unsafe {self.__real__.as_ref().unwrap()}}).unwrap());
                }
            }
        },
        Expr::Range(x) => {
            if x.from.is_some() {
                parse_expr(&mut x.from.as_mut().unwrap());
            }
            if x.to.is_some() {
                parse_expr(&mut x.to.as_mut().unwrap());
            }
        },
        Expr::Reference(x) => {
            parse_expr(&mut x.expr)
        },
        Expr::Repeat(x) => {
            parse_expr(&mut x.expr);
            parse_expr(&mut x.len);
        },
        Expr::Return(x) => {
            if x.expr.is_some() {
                parse_expr(&mut x.expr.as_mut().unwrap());
            }
        },
        Expr::Struct(x) => {
            for field in &mut x.fields {
                parse_expr(&mut field.expr);
            }
            if x.rest.is_some() {
                parse_expr(&mut x.rest.as_mut().unwrap());
            }
        },
        Expr::Try(x) => {
            parse_expr(&mut x.expr);
        },
        Expr::TryBlock(x) => {
            parse_block(&mut x.block);
        },
        Expr::Tuple(x) => {
            for elem in &mut x.elems {
                parse_expr(elem);
            }
        },
        Expr::Type(_) => todo!(),
        Expr::Unary(x) => {
            parse_expr(&mut x.expr);
        },
        Expr::Unsafe(x) => {
            parse_block(&mut x.block);
        },
        Expr::Verbatim(_) => todo!(),
        Expr::While(x) => {
            parse_expr(&mut x.cond);
            parse_block(&mut x.body);
        },
        Expr::Yield(x) => {
            if x.expr.is_some() {
                parse_expr(&mut x.expr.as_mut().unwrap());
            }
        },
        _ => todo!(),
    }
}
