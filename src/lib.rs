use std::{collections::HashMap, sync::{Mutex, RwLock, Arc}, cell::RefCell};

use parse_class::parse_class;
use proc_macro::TokenStream;
use lazy_static::lazy_static;
#[allow(unused_variables, dead_code, unused_mut, unused_imports)]
mod info;
#[allow(unused_variables, dead_code, unused_mut, unused_imports)]
mod parse_class;
#[allow(unused_variables, dead_code, unused_mut, unused_imports)]
mod parse_expr;
use info::{ClassInfo, ParentInfo};
use quote::quote;
use syn::{Type, TypeArray, parse2};

use crate::info::{SyncType, SyncClassInfo, Serializable};

fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

lazy_static!{
    static ref CLASSES: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
    //static ref t: Mutex<HashMap<String, SyncType>> = Mutex::new(HashMap::new());
}

#[proc_macro]
pub fn class(token: TokenStream) -> TokenStream {
    println!("before parse to ClassInfo");
    let mut class_info: ClassInfo = syn::parse(token).unwrap();
    let class_info = &mut class_info;
    println!("after parse to ClassInfo: {}", class_info.get_ident());

    println!("before parse_class");
    parse_class(class_info);
    println!("after parse_class");

    let _struct = class_info._struct.as_ref().unwrap();
    if class_info._impl.is_none() {
        panic!("there is no impl for this struct");
    }
    let _impl = class_info._impl.as_ref().unwrap();
    let _trait_impl = class_info._trait_impl.values();


    //let real = class_info.get_real();
    let name = class_info.get_ident();
    let _trait = class_info.real_trait.as_ref().unwrap();
    println!("before insert data to CLASSES");
    CLASSES.lock().as_mut().unwrap().insert(name.to_string(), class_info.serialize());
    println!("after insert data to CLASSES");
    
    quote!{
        #_trait
        #_struct
        #_impl
        //impl #real for #name {}
        #(#_trait_impl)*
    }.into()
}