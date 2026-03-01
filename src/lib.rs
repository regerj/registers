use syn::{ItemStruct, parse_macro_input};

use crate::register::Register;

mod common;
mod register;

#[derive(Debug, darling::FromMeta)]
#[darling(derive_syn_parse)]
struct MacroArgs {
    size: usize,
}

#[derive(darling::FromDeriveInput)]
#[darling(attributes(register), forward_attrs(allow, doc, cfg))]
struct RegisterOpts {
    //ident: syn::Ident,
    //attrs: Vec<syn::Attribute>,
    size: usize,
}

#[proc_macro_attribute]
pub fn register(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let strct = parse_macro_input!(item as ItemStruct);
    let reg = Register::new(strct);
    reg.implement().into()
}
