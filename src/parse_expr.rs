use proc_macro2::TokenStream;
use syn::{Expr, Block, Pat, Stmt, ExprTuple, Error};
use quote::{quote, ToTokens};

fn parse_statement(stmt: &mut Stmt) -> TokenStream {
    match stmt {
        Stmt::Local(x) => {
            if x.init.is_some() {
                parse_expr(&mut x.init.as_mut().unwrap().1);
            }
        },
        Stmt::Item(_) => { },
        Stmt::Expr(x) => {
            parse_expr(x);
        },
        Stmt::Semi(x, _) => {
            parse_expr(x);
        },
    }
    stmt.to_token_stream()
}

fn parse_pattern(pat: &mut Pat) -> TokenStream {
    pat.to_token_stream()
}

pub fn parse_block(block: &mut Block) -> TokenStream {
    for line in &mut block.stmts {
        parse_statement(line);
    }
    block.to_token_stream()
}

pub fn parse_expr(expr: &mut Expr) -> TokenStream{
    
    match expr {
        Expr::Array(x) => {
            for elem in &mut x.elems {
                parse_expr(elem);
            }
        },
        Expr::Assign(x) => { 
            parse_expr(&mut x.left);
            parse_expr(&mut x.right); 
        },
        Expr::AssignOp(x) => { 
            parse_expr(&mut x.left);
            parse_expr(&mut x.right); 
        },
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
        Expr::Break(_) => {},
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
        Expr::Continue(_) => {},
        Expr::Field(x) => {
            x.base = syn::parse2(parse_expr(&mut x.base)).unwrap();
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
            x.expr = syn::parse2(parse_expr(&mut x.expr)).unwrap();
            
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
            let tokens = &x.mac.tokens;
            
            let _expr: Result<ExprTuple, Error> = syn::parse2(quote!{( #tokens )});
            if _expr.is_ok() {
                let _expr: Result<ExprTuple, Error> = syn::parse2(parse_expr(&mut Expr::Tuple(_expr.unwrap().to_owned())));
                x.mac.tokens = _expr.unwrap().elems.to_token_stream();
            } else {
                let _expr: &mut Expr = &mut syn::parse2(tokens.clone()).unwrap();
                let _expr: Expr = syn::parse2(parse_expr(_expr)).unwrap();
                x.mac.tokens = _expr.to_token_stream();
            }
        },
        Expr::Match(x) => {
            parse_expr(&mut x.expr);
            for arm in &mut x.arms {
                parse_pattern(&mut arm.pat);
                parse_expr(&mut arm.body);
            }
        },
        Expr::MethodCall(method) => {
            method.receiver = syn::parse2(parse_expr(&mut method.receiver)).unwrap();
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
                let ident = segments[0].ident.to_string();
                if &ident == "this" {
                    return quote!{self}
                } else if &ident == "self" {
                    return quote!{unsafe {self.__real__.as_ref().unwrap()}}
                } else if &ident == "self_mut" {
                    return quote!{unsafe { self.__real__.as_mut().unwrap() }}
                } else if &ident == "_super" {
                    return quote!{self.__prototype__}
                } else if &ident == "_super_mut" {
                    return quote!{unsafe { self.__prototype__.as_mut().get_unchecked_mut() }}
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
            parse_expr(&mut x.expr);
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
        Expr::Type(_) => {},
        Expr::Unary(x) => {
            parse_expr(&mut x.expr);
        },
        Expr::Unsafe(x) => {
            parse_block(&mut x.block);
        },
        Expr::Verbatim(_) => {},
        Expr::While(x) => {
            parse_expr(&mut x.cond);
            parse_block(&mut x.body);
        },
        Expr::Yield(x) => {
            if x.expr.is_some() {
                parse_expr(&mut x.expr.as_mut().unwrap());
            }
        },
        _ => {
        },
    }
    expr.to_token_stream()
}
