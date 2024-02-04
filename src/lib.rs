//! Macro to implement inherit
//! 
//! Provide <code>class</code> to cover a struct itself, it's impl and trait implements that need to inherit.
//! To borrow as mut, see <code>def_as_mut</code>
//! 

use std::{collections::HashMap, sync::Mutex};

use parse_class::parse_class;
use proc_macro::TokenStream;
use lazy_static::lazy_static;
use quote::quote;

mod info;
mod parse_class;
mod parse_expr;
use info::ClassInfo;

use crate::info::Serializable;
lazy_static!{
    static ref CLASSES: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

/// 
/// need to wrap struct and implements.
/// 
/// using <code>#\[keep\]</code> for method make that method keep in the original impl.
/// 
/// methods without <code>#\[keep\]</code> will be put into trait <code>\_\_XXX\_\_</code>.
/// 
/// and macro will auto generate a <code>new</code> function which return <code>Pin<Box<Self>></code>.
/// 
/// expression in the method will be converted.
/// 
/// instead of use <code>self</code>, using <code>this</code>.
/// 
/// <code>self</code> will be convert to use <code>unsafe { self.\_\_real\_\_.as_ref().unwrap() }</code>.
/// 
/// <code>self_mut</code> will be convert to use <code>unsafe { self.\_\_real\_\_.as_mut().unwrap() }</code>.
/// 
/// <code>\_super</code> will be convert to use <code>self.\_\_prototype\_\_</code>.
/// 
/// <code>\_super_mut</code> will be convert to use <code>unsafe { self.\_\_prototype\_\_.as_mut().get_unchecked_mut() }</code>.
/// 
/// 
/// An example to use this macro:
/// ```rust
/// class! {
///     struct Example {
///         data: String    
///     }
///     impl Example {
///         #[keep]
///         fn with() -> Pin<Box<Self>> where Self: Sized {
///             Self::new("example".to_string())
///         }
///         
///         fn set_data(&mut self, data: String) {
///             this.data = data;
///         }
/// 
///         fn get_data(&self) -> String {
///             this.data.clone()
///         }
///     }
///     impl Something for Example {
///         //do something
///     }
/// }
/// class! {
///     extends Example;
///     struct Sub { }
///     impl Sub { }
/// }
/// ```
/// the struct will become:
/// ```rust
/// struct Example {
///     __real__: *mut dyn __Example__,
///     _pinned: ::std::marker::PhantomPinned,
///     data: String
/// }
/// struct Sub {
///     __prototype__: ::std::pin::Pin<Box<Example>>
///     __real__: *mut dyn __Sub__,
///     _pinned: ::std::marker::PhantomPinned,
/// }
/// ```
/// 

#[proc_macro]
pub fn class(token: TokenStream) -> TokenStream {
    let option =syn::parse(token);
    if option.is_err() {
        return option.err().unwrap().into_compile_error().into()
    }
    let mut class_info: ClassInfo = option.unwrap();
    let class_info = &mut class_info;

    parse_class(class_info);

    let _struct = class_info._struct.as_ref().unwrap();
    if class_info._impl.is_none() {
        panic!("there is no impl for this struct");
    }
    let _impl = class_info._impl.as_ref().unwrap();
    let _trait_impl = class_info._trait_impl.values();

    let name = class_info.get_ident();
    let _trait = class_info.real_trait.as_ref().unwrap();
    CLASSES.lock().as_mut().unwrap().insert(name.to_string(), class_info.serialize());
    
    quote!{
        #_trait
        #_struct
        #_impl
        #(#_trait_impl)*
    }.into()
}


/// this macro will define macro <code>as_mut</code>
/// 
/// example:
/// ```rust
/// def_as_mut!();
/// 
/// fn main() {
///     let mut example = Sub::new("data".to_string());
///     as_mut!(example).set_data("modified".to_string());
///     assert_eq!(example.get_data(), "modified".to_string());
/// }
/// ```


#[proc_macro]
pub fn def_as_mut(_: TokenStream) -> TokenStream {
    quote!{
        macro_rules! as_mut {
            ($tt: tt) => {
                unsafe {Pin::get_unchecked_mut( $tt .as_mut())}
            };
        }
    }.into()
}