use darling::{FromMeta, ast::NestedMeta};
use syn::{ItemStruct, parse_macro_input};

use crate::register::Register;

mod common;
mod register;

const SUPPORTED_WIDTHS: [usize; 4] = [8usize, 16usize, 32usize, 64usize];

#[derive(Debug, darling::FromMeta)]
#[darling(and_then = verify_args)]
struct MacroArgs {
    size: usize,

    #[darling(default = || true)]
    read: bool,

    #[darling(default = || true)]
    write: bool,
}

fn verify_args(args: MacroArgs) -> darling::Result<MacroArgs> {
    if !SUPPORTED_WIDTHS.contains(&args.size) {
        return Err(darling::Error::custom(format!(
            "Invalid size: {}, expected one of: {:?}",
            args.size, SUPPORTED_WIDTHS
        ))
        .with_span(&args.size));
    }

    Ok(args)
}

#[proc_macro_attribute]
pub fn register(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let nested_meta = match NestedMeta::parse_meta_list(_attr.into()) {
        Ok(v) => v,
        Err(e) => return e.to_compile_error().into(),
    };
    let macro_args = match MacroArgs::from_list(&nested_meta) {
        Ok(v) => v,
        Err(e) => return e.write_errors().into(),
    };
    let strct = parse_macro_input!(item as ItemStruct);
    let reg = Register::new(strct, macro_args);
    reg.implement().into()
}

#[cfg(test)]
mod tests {}
