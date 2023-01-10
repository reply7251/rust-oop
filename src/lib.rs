use std::{collections::HashMap, sync::Mutex};

use parse_class::parse_class;
use proc_macro::TokenStream;
use lazy_static::lazy_static;
mod info;
mod parse_class;
mod parse_expr;
use info::ClassInfo;
use quote::quote;

use crate::info::Serializable;

lazy_static!{
    static ref CLASSES: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
    //static ref t: Mutex<HashMap<String, SyncType>> = Mutex::new(HashMap::new());
}

#[proc_macro]
pub fn class(token: TokenStream) -> TokenStream {
    let mut class_info: ClassInfo = syn::parse(token).unwrap();
    let class_info = &mut class_info;

    parse_class(class_info);


    let _struct = class_info._struct.as_ref().unwrap();
    if class_info._impl.is_none() {
        panic!("there is no impl for this struct");
    }
    let _impl = class_info._impl.as_ref().unwrap();
    let _trait_impl = class_info._trait_impl.values();


    //let real = class_info.get_real();
    let name = class_info.get_ident();
    let _trait = class_info.real_trait.as_ref().unwrap();
    CLASSES.lock().as_mut().unwrap().insert(name.to_string(), class_info.serialize());
    
    quote!{
        #_trait
        #_struct
        #_impl
        //impl #real for #name {}
        #(#_trait_impl)*
    }.into()
}