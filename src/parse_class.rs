use std::borrow::Borrow;

use proc_macro::{TokenStream, Ident, Span};
use quote::{ToTokens, TokenStreamExt, quote};
use syn::{self, Token, ItemStruct, ItemImpl, parse::{Parse, Parser}, Result, Error, Path, Type, Field, Expr, Stmt, ExprBlock, Block, Pat, PathSegment, parse2, Signature, ImplItemMethod};

use crate::{info::ClassInfo, CLASSES};


fn get_methods(item_impl: &ItemImpl) -> Vec<syn::ImplItemMethod> {
    item_impl.items.iter().filter_map(|item| match item {
        syn::ImplItem::Method(x) => Some(x.clone()),
        _ => None,
    }).collect()
}

fn move_impl_to_real(item_impl: &mut ItemImpl, info: &mut ClassInfo) {
    let is_trait_impl = item_impl.trait_.is_some();
    if is_trait_impl {
        panic!("this should not be a trait impl");
    }

    let methods = get_methods(item_impl);
    let real = info.get_real();
    let name = info.get_ident();
    
    info._trait_impl.insert(Box::new(info.get_real()), Box::new(syn::parse2(quote!{
        impl #real for #name {
            #(#methods)*
        }
    }).unwrap()));
}

fn clear_methods(item_impl: &mut ItemImpl) {
    item_impl.items.retain(|x| match x {
        syn::ImplItem::Method(_) => false,
        _ => true,
    })
}

fn create_new(info: &mut ClassInfo) {
    info._impl.as_mut().unwrap().items.push(syn::ImplItem::Method(syn::parse2(quote!{
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
        /*
        move_methods_to_real
        for this : trait {
            tm = get_methods(this)
            for method : tm {
                replace_self(method)
            }
            clear_methods(this)
            add_methods(this, tm)
        }
        create constructor
        */
        create_new(info);
        //parse_impl(&mut info._impl.as_mut().unwrap());
    }
}



fn parse_impl_with_parent(item_impl: &mut ItemImpl, info: &ClassInfo, parent: &ClassInfo) {
    /*
        move_methods_to_prototype
        move_methods_to_real
        for this : trait {
            pm = get_methods(prototype)
            tm = get_methods(this)
            for method : tm {
                replace_self(method)
                replace_super(method)
                pm.remove(method)
            }
            clear_methods(this)
            add_methods(this, tm)
            for method : pm {
                parse_call_super(method)
            }
            add_methods(this, pm)
        }
        create constructor
    */
}

fn get_signature_string(method: &ImplItemMethod) -> String {
    method.sig.to_token_stream().to_string()
}

fn move_methods_to_prototype(info: &mut ClassInfo){
    let parent_info = info.get_parent_info();
    let prototype = parent_info.get().get_real();
    let name = info.get_ident();

    let mut o_prototype_impl = info._trait_impl.get_mut(&Box::new(info.get_real()));
    if o_prototype_impl.is_none() {
        create_prototype(info);
        o_prototype_impl = info._trait_impl.get_mut(&Box::new(info.get_real()));
    }
    
    let prototype_impl: &mut ItemImpl = o_prototype_impl.as_mut().unwrap().as_mut();
    let prototype_methods: Vec<String> = get_methods(prototype_impl).iter().map(get_signature_string).collect();
    
    let mut removed: Vec<String> = Vec::new();

    for method in get_methods(info._impl.as_ref().unwrap()) {
        let signature_string = get_signature_string(&method);
        if prototype_methods.iter().position(|x| *x == signature_string).is_some() {
            prototype_impl.items.push(syn::ImplItem::Method(method));
            removed.push(signature_string);
        }
    }
    info._impl.as_mut().unwrap().items.retain(|item| match item {
        syn::ImplItem::Method(method) => {
            !removed.contains(&get_signature_string(method))
        },
        _ => true,
    })
}

fn create_prototype(info: &mut ClassInfo) {
    let parent_info = info.get_parent_info();
    let prototype = parent_info.get().get_real();
    let name = info.get_ident();
    
    
    info._trait_impl.insert(Box::new(info.get_real()), Box::new(syn::parse2(quote!{
        impl #prototype for #name { }
    }).unwrap()));
}