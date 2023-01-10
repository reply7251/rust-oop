use std::{collections::HashMap};

use proc_macro2::Ident;
use quote::{ToTokens};
use syn::{self, Token, ItemStruct, ItemImpl, Result, ItemTrait};

use crate::CLASSES;
mod kw {
    syn::custom_keyword!(extends);
}

pub trait Serializable {
    fn serialize(&self) -> String;
    fn deserialize(from: String) -> Self;
}

pub struct ParentInfo {
    extend_token: kw::extends,
    pub parent: Ident,
    end: Option<Token![;]>,
}

impl syn::parse::Parse for ParentInfo {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let extend_token: kw::extends = input.parse()?;
        let parent: Ident = input.parse()?;
        let lookahead = input.lookahead1();
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

impl Clone for ParentInfo {
    fn clone(&self) -> Self {
        ParentInfo { 
            extend_token: self.extend_token.clone(), 
            parent: self.parent.clone(), 
            end: self.end.clone() 
        }
    }
}

impl Serializable for ParentInfo {
    fn serialize(&self) -> String {
        format!("extends {} ", self.parent)
    }
    fn deserialize(from: String) -> Self {
        syn::parse2(quote::quote!{extends #from ;}).unwrap()
    }
}

pub struct ClassInfo {
    pub _parent: Option<ParentInfo>,
    pub _struct: Option<ItemStruct>,
    pub _impl: Option<ItemImpl>,
    pub _trait_impl: HashMap<Box<syn::Ident>, Box<ItemImpl>>,
    pub real_trait: Option<ItemTrait>,
}

impl ClassInfo {
    pub fn get_real(&self) -> syn::Ident {
        syn::Ident::new(&format!("__{}__", self._struct.as_ref().unwrap().ident), proc_macro2::Span::call_site())
    }

    pub fn get_ident(&self) -> syn::Ident {
        self._struct.as_ref().unwrap().ident.clone()
    }

    pub fn get_parent_info(&self) -> ClassInfo {
        let class_map = CLASSES.lock().unwrap();
        //println!("before find parent");
        let result = class_map.get(&self._parent.as_ref().unwrap().parent.to_string());
        //println!("after find parent");
        if result.is_none() {
            panic!("the parent of {}, {} is null", self.get_ident(), self._parent.as_ref().unwrap().parent.to_string());
        } else {
            //println!("find parent: {}", self._parent.as_ref().unwrap().parent.to_string());
        }
        ClassInfo::deserialize(result.unwrap().to_string())
    }

    pub fn get_mro(&self) -> Vec<ClassInfo> {
        let class_map = CLASSES.lock().unwrap();
        let mut current = self.clone();
        let mut mro = Vec::new();
        loop {
            if current._parent.is_none() {
                return mro;
            }
            let result = class_map.get(&current._parent.as_ref().unwrap().parent.to_string());
            if result.is_none() {
                return mro
            } else {
                current = ClassInfo::deserialize(result.unwrap().to_string());
                mro.push(current.clone());
            }
        }
        
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
                _trait_impl.insert(Box::new(item_impl.trait_.as_ref().unwrap().1.get_ident().unwrap().clone()), item_impl);
            } else {
                _impl = Some(*item_impl);
            }
        }

        Ok(Self {
            _parent,
            _struct,
            _impl,
            _trait_impl,
            real_trait: None
        })
    }
}

impl Clone for ClassInfo {
    fn clone(&self) -> Self {
        ClassInfo { 
            _parent: self._parent.clone(), 
            _struct: self._struct.clone(), 
            _impl: self._impl.clone(), 
            _trait_impl: self._trait_impl.clone(), 
            real_trait: self.real_trait.clone()
        }
    }
}

impl Serializable for ClassInfo {
    fn deserialize(from: String) -> Self {
        syn::parse_str(&from).unwrap()
    }
    fn serialize(&self) -> String {
        let mut result = String::new();
        if self._parent.is_some() {
            result += &self._parent.as_ref().unwrap().serialize();
        }
        result += &self._struct.as_ref().unwrap().to_token_stream().to_string();
        result += &self._impl.as_ref().unwrap().to_token_stream().to_string();
        for _trait_impl in self._trait_impl.values() {
            result += &_trait_impl.to_token_stream().to_string();
        }
        result
    }
}

pub struct SyncType(pub Ident);

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

pub struct SyncClassInfo(pub ClassInfo);

unsafe impl Sync for SyncClassInfo {}
unsafe impl Send for SyncClassInfo {}

impl Clone for SyncClassInfo {
    fn clone(&self) -> Self {
        SyncClassInfo(self.0.clone())
    }
}