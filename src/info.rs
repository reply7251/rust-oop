use std::collections::HashMap;

use proc_macro::TokenStream;
use quote::{ToTokens, TokenStreamExt};
use syn::{self, Token, ItemStruct, ItemImpl, parse::Parse, Result, Error, Path, Type};

use crate::CLASSES;
mod kw {
    syn::custom_keyword!(extends);
}

#[derive(Clone)]
pub struct ParentInfo {
    extend_token: kw::extends,
    pub parent: Type,
    end: Option<Token![;]>,
}

impl syn::parse::Parse for ParentInfo {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        let extend_token: kw::extends = input.parse()?;
        let parent: Type = input.parse()?;
        let end: Option<Token![;]> = if lookahead.peek(Token![;]) {
            Some(input.parse()?)
        } else {
            None
        };
        Ok(Self {
            extend_token,
            parent,
            end
        })
    }
}

#[derive(Clone)]
pub struct ClassInfo {
    pub _parent: Option<ParentInfo>,
    pub _struct: Option<ItemStruct>,
    pub _impl: Option<ItemImpl>,
    pub _trait_impl: HashMap<Box<syn::Ident>, Box<ItemImpl>>,
}

impl ClassInfo {
    pub fn get_real(&self) -> syn::Ident {
        syn::Ident::new(&format!("__{}__", self._struct.as_ref().unwrap().ident), proc_macro2::Span::call_site())
    }

    pub fn get_ident(&self) -> syn::Ident {
        self._struct.as_ref().unwrap().ident.clone()
    }

    pub fn get_parent_info(&self) -> SyncClassInfo {
        let class_map = CLASSES.lock().unwrap();
        let result = class_map.get(&SyncType(self._parent.as_ref().unwrap().parent.clone()));
        if result.is_none() {
            panic!("the parent of {} is null", self.get_ident());
        }
        result.unwrap().clone()
    }
}

impl syn::parse::Parse for ClassInfo {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        let _parent: Option<ParentInfo> = if lookahead.peek(kw::extends) {
            Some(input.parse()?)
        } else {
            None
        };

        let _struct: Option<ItemStruct> = Some(input.parse()?);
        let mut _impl: Option<ItemImpl> = None;
        let mut _trait_impl: HashMap<Box<proc_macro2::Ident>, Box<ItemImpl>> = HashMap::new();

        while !input.is_empty() {
            let item_impl: Box<ItemImpl> = Box::new(input.parse()?);
            if item_impl.as_ref().trait_.is_some() {
                //let a = item_impl.as_ref().trait_.as_ref().unwrap().1.get_ident().as_ref().unwrap();
                _trait_impl.insert(Box::new(item_impl.trait_.as_ref().unwrap().1.get_ident().unwrap().clone()), item_impl);
            } else {
                _impl = Some(*item_impl);
            }
        }

        Ok(Self {
            _parent,
            _struct,
            _impl,
            _trait_impl
        })
    }
}
pub struct SyncType(Type);

unsafe impl Sync for SyncType {}
unsafe impl Send for SyncType {}
impl std::hash::Hash for SyncType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_token_stream().to_string().hash(state);
    }
}

impl PartialEq for SyncType {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_token_stream().to_string() == other.0.to_token_stream().to_string()
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl Eq for SyncType {
    fn assert_receiver_is_total_eq(&self) {
        
    }
}

#[derive(Clone)]
pub struct SyncClassInfo(ClassInfo);

unsafe impl Sync for SyncClassInfo {}
unsafe impl Send for SyncClassInfo {}

impl SyncClassInfo {
    pub fn get(&self) -> &ClassInfo{
        &self.0
    }
}

/* impl std::hash::Hash for SyncClassInfo {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        if self.0._impl.is_some() {
            self.0._impl.unwrap().to_token_stream().to_string().hash(state)
        }
        if self.0._parent.is_some() {
            self.0._parent.unwrap().parent.to_token_stream().to_string().hash(state)
        }
        if self.0._struct.is_some() {
            self.0._struct.unwrap().to_token_stream().to_string().hash(state)
        }
        for _trait in self.0._trait_impl.keys() {
            _trait.hash(state);
        }
    }
} */