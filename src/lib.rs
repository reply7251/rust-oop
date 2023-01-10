use std::{collections::HashMap, sync::{Mutex, RwLock, Arc}, cell::RefCell};

use parse_class::parse_class;
use proc_macro::TokenStream;
use lazy_static::lazy_static;
#[allow(unused_variables, dead_code, unused_mut, unused_imports)]
mod info;
#[allow(unused_variables, dead_code, unused_mut, unused_imports)]
mod parse_class;
use info::{ClassInfo, ParentInfo};
use quote::quote;
use syn::{Type, TypeArray, parse2};

use crate::info::{SyncType, SyncClassInfo};

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
    static ref CLASSES: Mutex<HashMap<SyncType, SyncClassInfo>> = Mutex::new(HashMap::new());
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

    let real = class_info.get_real();
    let name = class_info.get_ident();
    
    quote!{
        trait #real {}
        #_struct
        #_impl
        impl #real for #name {}
        #(#_trait_impl)*
    }.into()
}