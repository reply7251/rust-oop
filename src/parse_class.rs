use proc_macro2::Ident;
use quote::{ToTokens, quote};
use syn::{self, ItemImpl, parse::Parser, Field, Pat, ImplItemMethod, TraitItem, ImplItem, FnArg, FieldValue};

use crate::{info::ClassInfo, parse_expr};

fn get_methods(item_impl: &ItemImpl) -> Vec<syn::ImplItemMethod> {
    item_impl.items.iter().filter_map(|item| match item {
        syn::ImplItem::Method(x) => Some(x.clone()),
        _ => None,
    }).collect()
}

fn get_meta_from_method(method: &ImplItemMethod) -> Vec<String> {
    method.attrs.iter().filter_map(|x| x.parse_meta().ok()).map(|x| x.to_token_stream().to_string()).collect()
}

fn remove_meta_from_method(method: &mut ImplItemMethod, meta: &String) {
    let pos = method.attrs.iter_mut().position(|x| {
        let parsed = x.parse_meta();
        if parsed.is_ok() {
            &parsed.unwrap().to_token_stream().to_string() == meta
        }else {
            false
        }
    });
    if pos.is_some() {
        method.attrs.remove(pos.unwrap());
    }
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
        pub fn new( #(#inputs),* ) -> ::std::pin::Pin<Box<Self>> {
            let mut this = Box::pin(Self { 
                __real__: ::std::ptr::null_mut::<Self>(), 
                _pinned: ::std::marker::PhantomPinned,
                #(#fields),*
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
                        FnArg::Receiver(_) => {},
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
            _ => {},
        }
    }

    let parent_type = parent.get_ident();

    let rhs = quote!{
        .as_mut().get_unchecked_mut().__real__ =  this.as_mut().get_unchecked_mut();
    };
    let mut lhs = quote!{
        this
    };
    let mut real_setter: Vec<proc_macro2::TokenStream> = Vec::new();
    for _ in info.get_mro() {
        lhs = quote!{
            #lhs .as_mut().get_unchecked_mut().__prototype__
        };
        real_setter.push(quote!{
            #lhs #rhs
        });
    }

    let new_inputs = if parent_inputs.len() > 0 {
        quote!{ #(#parent_inputs),* ,  #(#inputs),* }
    } else {
        quote!{ #(#inputs),* }
    };

    info._impl.as_mut().unwrap().items.push(syn::ImplItem::Method(syn::parse2(quote!{
        pub fn new( #new_inputs ) -> ::std::pin::Pin<Box<Self>> {
            let __prototype__ = #parent_type ::new( #(#parent_inputs_call),* );
            let mut this = Box::pin(Self { 
                __prototype__,
                __real__: ::std::ptr::null_mut::<Self>(), 
                _pinned: ::std::marker::PhantomPinned,
                #(#fields),*
            });
            unsafe { 
                this.as_mut().get_unchecked_mut().__real__ =  this.as_mut().get_unchecked_mut();
                #(#real_setter);*
            };
            this
        }
    }).unwrap()));
}

pub fn parse_class(info: &mut ClassInfo) {
    let has_parent = info._parent.is_some();
    let parent: Option<Ident> = if has_parent {
        Some(info._parent.as_ref().unwrap().parent.clone())
    } else {
        None
    };
    if has_parent {
        let p = &info.get_parent_info();
        parse_impl_with_parent(info, p);
        create_new_with_parent(info, p);
    } else {
        parse_impl(info);
        create_new(info)
    }

    let keep = String::from("keep");
    for item in &mut info._impl.as_mut().unwrap().items {
        match item {
            syn::ImplItem::Method(method) => {
                remove_meta_from_method(method, &keep)
            },
            _ => {},
        }
    }
    
    let mut _struct = info._struct.as_mut().unwrap();
    match _struct.fields {
        syn::Fields::Named(ref mut fields) => {
            if has_parent {
                let p = &parent.unwrap();
                fields.named.push(Field::parse_named.parse2(quote!{__prototype__: ::std::pin::Pin<Box<#p>>}).unwrap());
            }
            let real = syn::Ident::new(&format!("__{}__", _struct.ident), proc_macro2::Span::call_site());
            fields.named.push(Field::parse_named.parse2(quote!{__real__: *mut dyn #real}).unwrap());
            fields.named.push(Field::parse_named.parse2(quote!{_pinned: ::std::marker::PhantomPinned}).unwrap());
        },
        syn::Fields::Unnamed(_) => {
            panic!("not support struct ({}) with unnamed field.", _struct.ident.to_string());
        },
        syn::Fields::Unit => {
            panic!("not support None ({}).", _struct.ident.to_string());
        },
    }
}

fn parse_impl(info: &mut ClassInfo) {
    create_real_trait(info);
    move_methods_to_real(info);
}

fn parse_impl_with_parent(info: &mut ClassInfo, parent: &ClassInfo) {
    move_methods_to_prototype(info);
    parse_impl(info);
    retrieve_implements_from_parent(info, parent);
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
            let mut is_mut = false;
            for input in &sign.inputs {
                match input {
                    syn::FnArg::Receiver(x) => {
                        if x.mutability.is_some() {
                            is_mut = true;
                        }
                    },
                    syn::FnArg::Typed(x) => {
                        match *x.pat.clone() {
                            Pat::Ident(ident) => {
                                if ident.ident.to_string() != "self" {
                                    inputs.push(ident.ident);
                                }
                            },
                            _ => {},
                        }
                    },
                }
            }

            let _super = if is_mut {
                quote!{ unsafe { self.__prototype__.as_mut().get_unchecked_mut() } }
            } else {
                quote!{ self.__prototype__ }
            };

            let override_method: ImplItemMethod = syn::parse2(quote!{
                #(#attrs)*
                #sign {
                    #_super . #method_name ( #(#inputs),* )
                }
            }).unwrap();
            
            override_methods.push(override_method);
        }

        let mut mapped: Vec<ImplItem> = override_methods.iter().map(|x| ImplItem::Method(x.to_owned())).collect();
        sub_impl.items.append(&mut mapped);
        
        info._trait_impl.insert(_trait_ident.clone(), sub_impl);
    }

}

fn retrieve_implements_from_parent(info: &mut ClassInfo, parent: &ClassInfo) {
    let mro: Vec<Ident> = info.get_mro().iter().map(|ci| ci.get_real()).collect();
    for key in parent._trait_impl.keys() {
        if !mro.contains(&key) && !info._trait_impl.contains_key(key) {
            create_trait_impl(info, key, &info.get_ident());
        }
    }
}

fn create_real_trait(info: &mut ClassInfo) {
    let real = info.get_real();

    let real_methods = get_methods(info._impl.as_ref().unwrap());

    let mut trait_items: Vec<TraitItem> = Vec::new();
    let keep = String::from("keep");
    for method in &real_methods {
        let attrs = &method.attrs;
        if get_meta_from_method(method).contains(&keep) {
            continue;
        }
        let sig = &method.sig;
        trait_items.push(syn::parse2(quote!{
            #(#attrs)*
            #sig ;
        }).unwrap());
    }

    if info._parent.is_some() {
        let prototype = info.get_parent_info().get_real();
        info.real_trait = Some(syn::parse2(quote!{
            pub trait #real : #prototype {
                #(#trait_items)*
            }
        }).unwrap());
    } else {
        info.real_trait = Some(syn::parse2(quote!{
            pub trait #real {
                #(#trait_items)*
            }
        }).unwrap());
    }
    info.real_trait.as_mut().unwrap().generics = info._impl.as_ref().unwrap().generics.clone();
}

fn get_signature_string(method: &ImplItemMethod) -> String {
    method.sig.to_token_stream().to_string()
}

fn move_methods_to_prototype(info: &mut ClassInfo){
    let mro = info.get_mro();
    for parent_info in mro {
        let prototype = parent_info.get_real();
        let key = Box::new(prototype.clone());
        let mut o_prototype_impl = info._trait_impl.get_mut(&key);

        if o_prototype_impl.is_none() {
            create_prototype(info, &parent_info);
            o_prototype_impl = info._trait_impl.get_mut(&key);
        }
        
        move_methods_to_impl(info._impl.as_mut().unwrap(), o_prototype_impl.as_mut().unwrap()
                , parent_info._trait_impl.get(&key).unwrap());
    }
}

fn move_methods_to_real(info: &mut ClassInfo){
    let real = info.get_real();
    let name = info.get_ident();

    let key = Box::new(real.clone());
    let mut o_real_impl = info._trait_impl.get_mut(&key);
    if o_real_impl.is_none() {
        create_trait_impl(info, &real, &name);
        o_real_impl = info._trait_impl.get_mut(&key);

        let _trait_impl = o_real_impl.as_mut().unwrap();
        let parent_trait_impl = info._impl.as_ref().unwrap();
        _trait_impl.generics = parent_trait_impl.generics.clone();
        _trait_impl.unsafety = parent_trait_impl.unsafety.clone();
        _trait_impl.defaultness = parent_trait_impl.defaultness.clone();
    }

    let from = info._impl.as_mut().unwrap();
    let to = o_real_impl.unwrap();

    let keep = String::from("keep");
    let methods = &mut get_methods(from);
    from.items.retain(|item| match item {
        syn::ImplItem::Method(_) => false,
        _ => true,
    });
    for method in methods {
        let attrs = get_meta_from_method(method);
        parse_expr::parse_block(&mut method.block);
        if attrs.contains(&keep) {
            remove_meta_from_method(method, &keep);
            from.items.push(syn::ImplItem::Method(method.to_owned()))
        } else {
            to.items.push(syn::ImplItem::Method(method.to_owned()));
        }
    }
}

fn move_methods_to_impl(from: &mut ItemImpl, to: &mut ItemImpl, origin: &ItemImpl) {
    let real_methods: Vec<String> = get_methods(origin).iter()
            .map(get_signature_string).collect();
    
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

fn create_prototype(info: &mut ClassInfo, parent_info: &ClassInfo) {
    let prototype = parent_info.get_real();
    let name = info.get_ident();
    create_trait_impl(info, &prototype, &name);
    let key = Box::new(prototype.clone());
    let _trait_impl = info._trait_impl.get_mut(&key).unwrap();
    let parent_trait_impl = parent_info._trait_impl.get(&key).unwrap();
    _trait_impl.generics = parent_trait_impl.generics.clone();
    _trait_impl.unsafety = parent_trait_impl.unsafety.clone();
    _trait_impl.defaultness = parent_trait_impl.defaultness.clone();
}

fn create_trait_impl(info: &mut ClassInfo, _trait: &Ident, name: &Ident) {
    let _trait_impl = Box::new(syn::parse2(quote!{
        impl #_trait for #name { }
    }).unwrap());
    info._trait_impl.insert(Box::new(_trait.clone()), _trait_impl);
}