use darling::FromField;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, ItemFn, ItemImpl, ItemMod, ItemStruct, parse_quote};

use crate::{MacroArgs, field::Field, util::unsign};

pub struct Register {
    ident: Ident,
    fields: Vec<Field>,
    args: MacroArgs,
}

impl Register {
    pub fn new(item: ItemStruct, args: MacroArgs) -> Self {
        let ident = item.ident;
        let fields: Vec<_> = item
            .fields
            .iter()
            .map(|field| Field::from_field(field).unwrap())
            .collect();
        Self {
            ident,
            fields,
            args,
        }
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
        let eq_raw_impl = self.eq_raw_impl();
        let read_impl = self.read_impl();
        let write_impl = self.write_impl();
        let raw_impl = self.raw_impl();
        parse_quote! {
            mod #mod_ident {
                #struct_impl
                impl #ident {
                    #new_impl
                    #clear_impl
                    #( #get_impls )*
                    #( #set_impls )*
                    #raw_impl
                    #read_impl
                    #write_impl
                }
                #from_impl
                #into_impl
                #eq_raw_impl
            }
        }
    }

    fn new_impl(&self) -> ItemFn {
        parse_quote! {
            pub fn new() -> Self {
                Self { reg: 0 }
            }
        }
    }

    fn read_impl(&self) -> Option<ItemFn> {
        let inner_ty = self.inner_type();
        if self.args.read {
            Some(parse_quote! {
                pub unsafe fn read(&mut self, addr: *const #inner_ty) {
                    unsafe { self.reg = core::ptr::read_volatile(addr) }
                }
            })
        } else {
            None
        }
    }

    fn raw_impl(&self) -> ItemFn {
        let inner_ty = self.inner_type();
        parse_quote! {
            pub fn raw(&self) -> #inner_ty {
                self.reg
            }
        }
    }

    fn write_impl(&self) -> Option<ItemFn> {
        let inner_ty = self.inner_type();
        if self.args.write {
            Some(parse_quote! {
                pub unsafe fn write(&self, addr: *mut #inner_ty) {
                    unsafe { core::ptr::write_volatile(addr, self.reg) }
                }
            })
        } else {
            None
        }
    }

    fn get_impls(&self) -> Vec<ItemFn> {
        self.fields
            .iter()
            .filter_map(|f| {
                if f.read {
                    Some(f.get_impl(self.args.size))
                } else {
                    None
                }
            })
            .collect()
    }

    fn set_impls(&self) -> Vec<ItemFn> {
        self.fields
            .iter()
            .filter_map(|f| {
                if f.write {
                    Some(f.set_impl(self.args.size))
                } else {
                    None
                }
            })
            .collect()
    }

    fn struct_impl(&self) -> ItemStruct {
        let ident = self.ident.clone();
        let inner_ty = self.inner_type();
        parse_quote! {
            #[derive(Debug, Clone)]
            pub struct #ident {
                reg: #inner_ty,
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
    #[allow(clippy::wrong_self_convention)]
    fn from_impl(&self) -> ItemImpl {
        let ident = self.ident.clone();
        let inner_ty = self.inner_type();
        parse_quote! {
            impl From<#inner_ty> for #ident {
                fn from(value: #inner_ty) -> Self {
                    Self { reg: value }
                }
            }
        }
    }

    #[allow(clippy::wrong_self_convention)]
    fn into_impl(&self) -> ItemImpl {
        let ident = self.ident.clone();
        let inner_ty = self.inner_type();
        parse_quote! {
            impl Into<#inner_ty> for #ident {
                fn into(self) -> #inner_ty {
                    self.reg
                }
            }
        }
    }

    fn eq_raw_impl(&self) -> ItemImpl {
        let ident = self.ident.clone();
        let inner_ty = self.inner_type();
        parse_quote! {
            impl PartialEq<#inner_ty> for #ident {
                fn eq(&self, other: &#inner_ty) -> bool {
                    self.reg == *other
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

    fn inner_type(&self) -> syn::Type {
        unsign(self.args.size)
    }
}
