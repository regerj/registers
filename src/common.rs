use quote::format_ident;
use syn::{ItemFn, Type, parse_quote};

#[derive(darling::FromField)]
#[darling(attributes(field))]
pub struct Field {
    pub ident: Option<syn::Ident>,
    pub signed: bool,
    pub msb: usize,
    pub lsb: usize,
    #[darling(default = "_true")]
    pub write: bool,
    #[darling(default = "_true")]
    pub read: bool,
}

fn _true() -> bool {
    true
}

impl Field {
    pub fn get_impl(&self, size: usize) -> ItemFn {
        let ident = self.ident.clone().expect("Field ident expected");
        let ty = self.io_ty(size);
        let lsb = self.lsb;
        let msb = self.msb;
        let field_size = msb - lsb + 1;
        let field_mask = 2u32.pow(field_size as u32) - 1;

        let fn_ident = format_ident!("get_{ident}");
        parse_quote! {
            pub fn #fn_ident(&self) -> #ty {
                let end_trimmed = self.reg >> #lsb;
                end_trimmed & #field_mask
            }
        }
    }

    pub fn set_impl(&self, size: usize) -> ItemFn {
        let ty = self.io_ty(size);
        let ident = self.ident.clone().expect("Field ident expected");
        let fn_ident = format_ident!("set_{ident}");
        let field_size = self.msb - self.lsb + 1;
        let field_max = 2u32.pow(field_size as u32) - 1;
        let field_mask = field_max << self.lsb;
        let lsb = self.lsb;
        parse_quote! {
            pub fn #fn_ident(&mut self, val: #ty) -> std::result::Result<(), String> {
                if val > #field_max {
                    return Err("".to_string());
                }
                self.reg = self.reg & !#field_mask;
                let val = val << #lsb;
                self.reg = self.reg | val;

                Ok(())
            }
        }
    }

    fn io_ty(&self, size: usize) -> Type {
        match size {
            8 => if self.signed { parse_quote!(i8) } else { parse_quote!(u8) }
            16 => if self.signed { parse_quote!(i16) } else { parse_quote!(u16) }
            32 => if self.signed { parse_quote!(i32) } else { parse_quote!(u32) }
            64 => if self.signed { parse_quote!(i64) } else { parse_quote!(u64) }
            _ => panic!("Invalid register size"),
        }
    }
}
