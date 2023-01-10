use proc_macro::{TokenStream, Ident, Span};
use quote::{ToTokens, TokenStreamExt, quote};
use syn::{self, Token, ItemStruct, ItemImpl, parse::{Parse, Parser}, Result, Error, Path, Type, Field, Expr, Stmt, ExprBlock, Block, Pat, PathSegment, parse2};

use crate::{info::ClassInfo, CLASSES};

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

fn get_methods(item_impl: &ItemImpl) -> Vec<syn::Ident> {
    item_impl.items.iter().filter_map(|item| match item {
        syn::ImplItem::Method(x) => Some(x.sig.ident.clone()),
        _ => None,
    }).collect()
}

fn parse_impl_with_parent(item_impl: &mut ItemImpl, parent: &ClassInfo) {
    let mut has_new = false;
    let is_trait_impl = item_impl.trait_.is_some();
    let mut parent_impl: Option<&ItemImpl> = None;
    let mut parent_methods: Vec<syn::Ident> = Vec::new();
    if is_trait_impl {
        let ident = item_impl.trait_.as_ref().unwrap().1.get_ident().unwrap();
        parent_impl = Some(parent._trait_impl.get(ident).unwrap());
    } else {
        parent_impl = Some(parent._impl.as_ref().unwrap());
    }
    
    for item in &mut item_impl.items {
        match item {
            syn::ImplItem::Const(_) => todo!(),
            syn::ImplItem::Method(method) => {
                
                parse_block(&mut method.block);
            },
            syn::ImplItem::Type(_) => todo!(),
            syn::ImplItem::Macro(_) => todo!(),
            syn::ImplItem::Verbatim(_) => todo!(),
            _ => todo!(),
        }
    }
}

fn parse_impl(item_impl: &mut ItemImpl) {
    item_impl.items.push(syn::ImplItem::Method(syn::parse2(quote!{
        pub fn new() -> Pin<Box<Self>> {
            let mut this = Box::pin(Self { 
                __real__: std::ptr::null_mut::<Self>(), 
                _pinned: std::marker::PhantomPinned,
            });
            unsafe { 
                this.as_mut().get_unchecked_mut().__real__ =  this.as_mut().get_unchecked_mut();
            };
            this
        }
    }).unwrap()));
}

pub fn parse_class(info: &mut ClassInfo) {
    //let info = info;
    let has_parent = info._parent.is_some();
    let parent: Option<Type> = if has_parent {
        Some(info._parent.as_ref().unwrap().parent.clone())
    } else {
        None
    };
    let mut _struct = info._struct.as_mut().unwrap();

    match _struct.fields {
        syn::Fields::Named(ref mut fields) => {
            if has_parent {
                let p = &parent.unwrap();
                fields.named.push(Field::parse_named.parse2(quote!{__prototype__: std::pin::Pin<Box<#p>>}).unwrap());
            }
            let real = syn::Ident::new(&format!("__{}__", _struct.ident), proc_macro2::Span::call_site());
            fields.named.push(Field::parse_named.parse2(quote!{__real__: *mut dyn #real}).unwrap());
            fields.named.push(Field::parse_named.parse2(quote!{_pinned: std::marker::PhantomPinned}).unwrap());
        },
        syn::Fields::Unnamed(_) => todo!(),
        syn::Fields::Unit => todo!(),
    }
    //let mut class_map = CLASSES.lock().unwrap();

    if has_parent {
        
    } else {
        parse_impl(&mut info._impl.as_mut().unwrap());
    }
}