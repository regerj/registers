use darling::{FromMeta, ast::NestedMeta};
use quote::quote;
use syn::{Attribute, Ident, ItemStruct, braced, parse::Parse, parse_macro_input, spanned::Spanned};

use crate::register::Register;

mod common;
mod register;

const SUPPORTED_WIDTHS: [usize; 4] = [ 8usize, 16usize, 32usize, 64usize ];

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

struct RegisterSyn {
    ident: Ident,
    fields: Vec<FieldSyn>,
}

struct FieldSyn {
    attr: Vec<syn::Attribute>,
    ident: Ident,
    signed: bool,
}

impl Parse for FieldSyn {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs: Vec<syn::Attribute> = input.call(Attribute::parse_outer)?;
        let ident: Ident = input.parse()?;
        let _: syn::token::Colon = input.parse()?;
        let ty: syn::Type = input.parse()?;
        let type_error = syn::Error::new(ty.span(), "Use \"u\" for unsigned and \"i\" for signed fields ");
        let signed = match ty {
            syn::Type::Path(path) => {
                if path.path.is_ident("u") {
                    false
                } else if path.path.is_ident("i") {
                    true
                } else {
                    return Err(type_error);
                }
            }
            _ => return Err(type_error)
        };

        Ok(Self { attr: attrs, ident, signed })
    }
}

impl Parse for RegisterSyn {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let _: syn::token::Struct = input.parse()?;
        let reg_ident: Ident = input.parse()?;
        let content;
        let _ = braced!(content in input);

        let foo = content.parse_terminated(FieldSyn::parse, syn::token::Comma)?;
        let foo: Vec<_> = foo.into_iter().collect();
        Ok(RegisterSyn { ident: reg_ident, fields: foo })
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

#[derive(Debug, darling::FromMeta)]
struct _MacroArgs {
    size: usize,

    #[darling(default = _true)]
    read: bool,

    #[darling(default = _true)]
    write: bool,
}

#[proc_macro_attribute]
pub fn _register (attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let meta = match NestedMeta::parse_meta_list(attr.into()) {
        Ok(v) => v,
        Err(e) => return e.to_compile_error().into()
    };
    let macro_args = match _MacroArgs::from_list(&meta) {
        Ok(v) => v,
        Err(e) => return e.write_errors().into()
    };

    if !SUPPORTED_WIDTHS.contains(&macro_args.size) {
        return syn::Error::new(macro_args.size.span(), format!("Supported widths: {SUPPORTED_WIDTHS:?}")).to_compile_error().into();
    }

    let reg = parse_macro_input!(item as RegisterSyn);
    //let reg = Register::new(reg, macro_args);
    //reg.implement().into()
    quote! {}.into()
}

#[cfg(test)]
mod tests {
    use darling::{FromMeta, ast::NestedMeta};
    use quote::quote;

    use crate::{_MacroArgs, FieldSyn, RegisterSyn};

    #[test]
    fn test_args() {
        let cut = quote! { read = false, write = true, size = 32 };
        let meta = NestedMeta::parse_meta_list(cut).expect("Failed to parse");
        let parsed = _MacroArgs::from_list(&meta).expect("Failed to parse");

        assert_eq!(parsed.size, 32);
        assert!(!parsed.read);
        assert!(parsed.write);
    }

    #[test]
    fn test_parse_field() {
        let cut = r"
            #[field(foo)]
            foo: u";
        let parsed: FieldSyn = syn::parse_str(cut).expect("Failed to parse");
        assert_eq!(parsed.ident, "foo");
        assert_eq!(parsed.signed, false);
    }

    #[test]
    fn test_parse_register() {
        let cut = r"
            struct HIF {
                #[field(foo)]
                foo: i,
                #[field(bar)]
                bar: u,
            }";

        let parsed: RegisterSyn = syn::parse_str(cut).expect("Failed to parse");
        assert_eq!(parsed.fields.len(), 2);

        assert_eq!(parsed.ident, "HIF");
        assert_eq!(parsed.fields[0].ident, "foo");
        assert!(parsed.fields[0].signed);
        assert_eq!(parsed.fields[1].ident, "bar");
        assert!(!parsed.fields[1].signed);
    }
}
