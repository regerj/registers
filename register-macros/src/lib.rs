use darling::{FromMeta, ast::NestedMeta};
use syn::{ItemStruct, parse_macro_input};

use crate::register::Register;

mod common;
mod register;

fn _true() -> bool {
    true
}

#[derive(Debug, darling::FromMeta)]
#[darling(default)]
struct MacroArgs {
    #[darling(default = _true)]
    read: bool,

    #[darling(default = _true)]
    write: bool,
}

impl Default for MacroArgs {
    fn default() -> Self {
        Self {
            read: true,
            write: true,
        }
    }
}

#[proc_macro_attribute]
pub fn register(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let nested_meta = NestedMeta::parse_meta_list(_attr.into()).unwrap();
    let macro_args = MacroArgs::from_list(&nested_meta).unwrap();
    let strct = parse_macro_input!(item as ItemStruct);
    let reg = Register::new(strct, macro_args);
    reg.implement().into()
}
