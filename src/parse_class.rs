use std::{borrow::Borrow, collections::HashMap};

use proc_macro::{TokenStream, Span};
use proc_macro2::Ident;
use quote::{ToTokens, TokenStreamExt, quote};
use syn::{self, Token, ItemStruct, ItemImpl, parse::{Parse, Parser}, Result, Error, Path, Type, Field, Expr, Stmt, ExprBlock, Block, Pat, PathSegment, parse2, Signature, ImplItemMethod, ItemTrait, TraitItem, ImplItem, FnArg, FieldValue};

use crate::{info::ClassInfo, CLASSES, parse_expr};


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
    let _struct = info._struct.as_ref().unwrap();
    let mut inputs: Vec<FnArg> = Vec::new();
    let mut fields: Vec<FieldValue> = Vec::new();
    for field in &_struct.fields {
        if field.ident.as_ref().is_some() {
            let id = field.ident.as_ref().unwrap().clone();
            let ty = field.ty.clone();
            
            inputs.push(syn::parse2(quote!{#id: #ty}).unwrap());
            fields.push(syn::parse2(quote!{#id: #id}).unwrap());
        } else {
            panic!("not implement for unnamed field!");
        }
    }

    info._impl.as_mut().unwrap().items.push(syn::ImplItem::Method(syn::parse2(quote!{
        pub fn new( #(#inputs),* ) -> Pin<Box<Self>> {
            let mut this = Box::pin(Self { 
                #(#fields),* ,
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

fn create_new_with_parent(info: &mut ClassInfo, parent: &ClassInfo) {
    let _struct = info._struct.as_ref().unwrap();
    let mut inputs: Vec<FnArg> = Vec::new();
    let mut parent_inputs: Vec<FnArg> = Vec::new();
    let mut parent_inputs_call: Vec<Ident> = Vec::new();
    let mut fields: Vec<FieldValue> = Vec::new();
    for field in &_struct.fields {
        if field.ident.as_ref().is_some() {
            let id = field.ident.as_ref().unwrap().clone();
            let ty = field.ty.clone();
            
            inputs.push(syn::parse2(quote!{#id: #ty}).unwrap());
            fields.push(syn::parse2(quote!{#id: #id}).unwrap());
        } else {
            panic!("not implement for unnamed field!");
        }
    }
    
    for item in &parent._impl.as_ref().unwrap().items {
        match item {
            ImplItem::Method(x) => {
                for arg in &x.sig.inputs {
                    parent_inputs.push(arg.clone());
                    match arg {
                        FnArg::Receiver(y) => {
                        },
                        FnArg::Typed(pat_type) => {
                            match *pat_type.pat.clone() {
                                Pat::Ident(ident) => {
                                    parent_inputs_call.push(ident.ident.clone());
                                },
                                _ => {
                                },
                            }
                        },
                    }
                }
            },
            _ => todo!(),
        }
    }

    //let __prototype__ = Animal::new(name);
    let parent_type = parent.get_ident();

    info._impl.as_mut().unwrap().items.push(syn::ImplItem::Method(syn::parse2(quote!{
        pub fn new( #(#parent_inputs),*  ,  #(#inputs),* ) -> Pin<Box<Self>> {
            let __prototype__ = #parent_type ::new( #(#parent_inputs_call),* );
            let mut this = Box::pin(Self { 
                #(#fields),* ,
                __prototype__,
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
    let parent: Option<Ident> = if has_parent {
        Some(info._parent.as_ref().unwrap().parent.clone())
    } else {
        None
    };
    //let mut class_map = CLASSES.lock().unwrap();

    println!("before parse_impl");
    if has_parent {
        println!("before get_parent_info");
        let p = &info.get_parent_info();
        println!("after get_parent_info");
        parse_impl_with_parent(info, p);
        create_new_with_parent(info, p);
    } else {
        parse_impl(info);
        create_new(info)
        //parse_impl(&mut info._impl.as_mut().unwrap());
    }
    println!("after parse_impl");
    
    println!("before refactor class");
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
    println!("after refactor class");
}

fn parse_impl(info: &mut ClassInfo) {
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
    println!("before create_real_trait");
    create_real_trait(info);
    println!("after create_real_trait");
    println!("before move_methods_to_real");
    move_methods_to_real(info);
    println!("after move_methods_to_real");
}

fn parse_impl_with_parent(info: &mut ClassInfo, parent: &ClassInfo) {
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
    
    println!("before move_methods_to_prototype");
    move_methods_to_prototype(info);
    println!("after move_methods_to_prototype");
    println!("before parse_impl inner");
    parse_impl(info);
    println!("after parse_impl inner");
    for _trait_ident in info._trait_impl.clone().keys() {
        let _trait_ident = _trait_ident.clone();
        let o_prototype = parent._trait_impl.get(&_trait_ident.clone());
        if o_prototype.is_none() {
            continue;
        }
        let mut prototype_methods = get_methods(o_prototype.unwrap());
        let mut sub_impl = info._trait_impl.get(&_trait_ident.clone()).unwrap().to_owned();
        let mut override_methods = get_methods(&sub_impl);

        for method in &mut override_methods {
            parse_expr::parse_block(&mut method.block);
            let find = prototype_methods.iter_mut().position(|x| get_signature_string(x) == get_signature_string(method));
            if find.is_some() {
                prototype_methods.remove(find.unwrap());
            }
        }
        
        clear_methods(sub_impl.as_mut());

        for method in &prototype_methods {
            let attrs = &method.attrs;
            let sign = &method.sig;
            let method_name = &method.sig.ident;
            let mut inputs: Vec<Ident> = Vec::new();
            for input in &sign.inputs {
                match input {
                    syn::FnArg::Receiver(_) => {},
                    syn::FnArg::Typed(x) => {
                        match *x.pat.clone() {
                            Pat::Ident(ident) => {
                                if ident.ident.to_string() != "self" {
                                    inputs.push(ident.ident);
                                }
                            },
                            _ => todo!(),
                        }
                    },
                }
            }
            
            override_methods.push(syn::parse2(quote!{
                #(#attrs)*
                #sign {
                    self.__prototype__. #method_name ( #(#inputs),* )
                }
            }).unwrap())
        }

        let mut mapped: Vec<ImplItem> = override_methods.iter().map(|x| ImplItem::Method(x.to_owned())).collect();

        sub_impl.items.append(&mut mapped);
        
        info._trait_impl.insert(_trait_ident.clone(), sub_impl);
    }

}

fn create_real_trait(info: &mut ClassInfo) {
    let real = info.get_real();

    let real_methods = get_methods(info._impl.as_ref().unwrap());

    let mut trait_items: Vec<TraitItem> = Vec::new();
    
    for method in &real_methods {
        let attrs = &method.attrs;
        let sig = &method.sig;
        trait_items.push(syn::parse2(quote!{
            #(#attrs)*
            #sig ;
        }).unwrap())
    }

    if info._parent.is_some() {
        let prototype = info.get_parent_info().get_real();
        info.real_trait = Some(syn::parse2(quote!{
            trait #real : #prototype {
                #(#trait_items)*
            }
        }).unwrap())
    } else {
        info.real_trait = Some(syn::parse2(quote!{
            trait #real {
                #(#trait_items)*
            }
        }).unwrap())
    }
}

fn get_signature_string(method: &ImplItemMethod) -> String {
    method.sig.to_token_stream().to_string()
}

fn move_methods_to_prototype(info: &mut ClassInfo){
    let parent_info = info.get_parent_info();
    let prototype = parent_info.get_real();
    println!("before get prototype");
    let mut o_prototype_impl = info._trait_impl.get_mut(&Box::new(prototype.clone()));
    println!("after get prototype");
    if o_prototype_impl.is_none() {
        
        println!("before create_prototype");
        create_prototype(info);
        println!("after create_prototype");
        o_prototype_impl = info._trait_impl.get_mut(&Box::new(prototype));
    }

    println!("before move_methods_to_impl");
    move_methods_to_impl(info._impl.as_mut().unwrap(), o_prototype_impl.as_mut().unwrap());
    println!("after move_methods_to_impl");
}

fn move_methods_to_real(info: &mut ClassInfo){
    let real = info.get_real();
    let name = info.get_ident();

    let mut o_real_impl = info._trait_impl.get_mut(&Box::new(real.clone()));
    if o_real_impl.is_none() {
        create_trait_impl(info, &real, &name);
        o_real_impl = info._trait_impl.get_mut(&Box::new(real));
    }

    let from = info._impl.as_mut().unwrap();
    let to = o_real_impl.unwrap();

    let real_methods: Vec<String> = get_methods(to).iter().map(get_signature_string).collect();

    for method in get_methods(from) {
        to.items.push(syn::ImplItem::Method(method));
    }
    from.items.retain(|item| match item {
        syn::ImplItem::Method(_) => false,
        _ => true,
    });
}

fn move_methods_to_impl(from: &mut ItemImpl, to: &mut ItemImpl) {
    let real_methods: Vec<String> = get_methods(to).iter().map(get_signature_string).collect();
    
    let mut removed: Vec<String> = Vec::new();

    for method in get_methods(from) {
        let signature_string = get_signature_string(&method);
        if real_methods.contains(&signature_string) {
            to.items.push(syn::ImplItem::Method(method));
            removed.push(signature_string);
        }
    }
    from.items.retain(|item| match item {
        syn::ImplItem::Method(method) => {
            !removed.contains(&get_signature_string(method))
        },
        _ => true,
    });
}

fn create_prototype(info: &mut ClassInfo) {
    let parent_info = info.get_parent_info();
    let prototype = parent_info.get_real();
    let name = info.get_ident();

    create_trait_impl(info, &prototype, &name);
}

fn create_trait_impl(info: &mut ClassInfo, _trait: &Ident, name: &Ident) {
    info._trait_impl.insert(Box::new(_trait.clone()), Box::new(syn::parse2(quote!{
        impl #_trait for #name { }
    }).unwrap()));
}