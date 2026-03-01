use darling::FromField;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, ItemFn, ItemImpl, ItemMod, ItemStruct, parse_quote};

use crate::common::Field;

pub struct Register {
    ident: Ident,
    fields: Vec<Field>,
}

impl Register {
    pub fn new(item: ItemStruct) -> Self {
        let ident = item.ident;
        let fields: Vec<_> = item
            .fields
            .iter()
            .map(|field| Field::from_field(field).unwrap())
            .collect();
        Self { ident, fields }
    }

    fn mod_impl(&self) -> ItemMod {
        let mod_ident = self.mod_ident();
        let struct_impl = self.struct_impl();
        let get_impls = self.get_impls();
        let set_impls = self.set_impls();
        let ident = self.ident.clone();
        let new_impl = self.new_impl();
        let from_impl = self.from_impl();
        let into_impl = self.into_impl();
        let clear_impl = self.clear_impl();
        parse_quote! {
            mod #mod_ident {
                #struct_impl
                impl #ident {
                    #new_impl
                    #clear_impl
                    #( #get_impls )*
                    #( #set_impls )*
                }
                #from_impl
                #into_impl
            }
        }
    }

    fn new_impl(&self) -> ItemFn {
        parse_quote! {
            pub fn new(reg: u32) -> Self {
                Self { reg }
            }
        }
    }

    fn get_impls(&self) -> Vec<ItemFn> {
        self.fields.iter().map(|f| f.get_impl(32)).collect()
    }

    fn set_impls(&self) -> Vec<ItemFn> {
        self.fields.iter().map(|f| f.set_impl(32)).collect()
    }

    fn struct_impl(&self) -> ItemStruct {
        let ident = self.ident.clone();
        parse_quote! {
            pub struct #ident {
                reg: u32,
            }
        }
    }

    fn clear_impl(&self) -> ItemFn {
        parse_quote! {
            pub fn clear(&mut self) {
                self.reg = 0;
            }
        }
    }

    fn from_impl(&self) -> ItemImpl {
        let ident = self.ident.clone();
        parse_quote! {
            impl From<u32> for #ident {
                fn from(value: u32) -> Self {
                    Self { reg: value }
                }
            }
        }
    }

    fn into_impl(&self) -> ItemImpl {
        let ident = self.ident.clone();
        parse_quote! {
            impl Into<u32> for #ident {
                fn into(self) -> u32 {
                    self.reg
                }
            }
        }
    }

    fn mod_ident(&self) -> Ident {
        format_ident!("_register_{}", self.ident.to_string().to_lowercase())
    }

    pub fn implement(&self) -> TokenStream {
        let mod_impl = self.mod_impl();
        let mod_ident = self.mod_ident();
        quote! {
            #mod_impl
            pub use #mod_ident::*;
        }
    }
}
