use quote::format_ident;
use syn::{Ident, ItemFn, Type, parse_quote};

#[derive(darling::FromField, Default)]
#[darling(attributes(field))]
pub struct Field {
    pub ident: Option<syn::Ident>,
    #[darling(default)]
    pub signed: bool,
    pub msb: usize,
    pub lsb: usize,
    #[darling(default = _true)]
    pub write: bool,
    #[darling(default = _true)]
    pub read: bool,
}

fn _true() -> bool {
    true
}

impl Field {
    pub fn get_impl(&self, size: usize) -> ItemFn {
        if self.signed {
            self.signed_get_impl(size)
        } else {
            self.unsigned_get_impl(size)
        }
    }

    fn signed_get_impl(&self, size: usize) -> ItemFn {
        assert!(self.signed);
        let fn_ident = self.get_fn_ident();
        let lsb = self.lsb;
        let msb = self.msb;
        let abs_field_mask = self.signed_field_mask_nosignbit();
        let ty = self.io_ty(size);

        parse_quote! {
            pub fn #fn_ident(&self) -> #ty {
                let signed = (self.reg >> #msb) & 1;
                if signed == 1 {
                    (!(#abs_field_mask >> #lsb) | self.reg & (#abs_field_mask >> #lsb)) as #ty
                } else {
                    ((self.reg & #abs_field_mask) >> #lsb) as #ty
                }
            }
        }
    }

    fn unsigned_get_impl(&self, size: usize) -> ItemFn {
        assert!(!self.signed);
        let fn_ident = self.get_fn_ident();
        let ty = self.io_ty(size);
        let lsb = self.lsb;
        let field_mask = self.field_mask();
        parse_quote! {
            pub fn #fn_ident(&self) -> #ty {
                (self.reg & #field_mask) >> #lsb
            }
        }
    }

    pub fn set_impl(&self, size: usize) -> ItemFn {
        if self.signed {
            self.signed_set_impl(size)
        } else {
            self.unsigned_set_impl(size)
        }
    }

    fn signed_field_mask_nosignbit(&self) -> u32 {
        assert!(self.signed);
        self.field_mask() & !(1 << self.msb)
    }

    fn unsigned_set_impl(&self, size: usize) -> ItemFn {
        assert!(!self.signed);
        let ty = self.io_ty(size);
        let fn_ident = self.set_fn_ident();
        let field_max = self.field_max();
        let field_mask = self.field_mask();
        let lsb = self.lsb;
        parse_quote! {
            pub fn #fn_ident(&mut self, val: #ty) -> registers::Result<()> {
                if val > #field_max {
                    return Err(registers::Error::OutOfBoundsFieldWrite);
                }

                self.reg = self.reg & !(#field_mask);
                self.reg = self.reg | (val << #lsb);

                Ok(())
            }
        }
    }

    fn signed_set_impl(&self, size: usize) -> ItemFn {
        assert!(self.signed);
        let ty = self.io_ty(size);
        let fn_ident = self.set_fn_ident();
        let field_mask = self.field_mask();
        let abs_field_mask = self.signed_field_mask_nosignbit();
        let msb = self.msb;
        let lsb = self.lsb;
        let field_max = self.field_max();
        let field_min = self.field_min();

        parse_quote! {
            pub fn #fn_ident(&mut self, mut val: #ty) -> registers::Result<()> {
                if val > #field_max as i32 || val < #field_min {
                    return Err(registers::Error::OutOfBoundsFieldWrite);
                }

                let signed_bit = if val < 0 {
                    1 << #msb
                } else {
                    0
                };

                self.reg = self.reg & !(#field_mask);
                self.reg = self.reg | signed_bit | ((val as u32 & (#abs_field_mask >> #lsb)) << #lsb);

                Ok(())
            }
        }
    }

    fn get_fn_ident(&self) -> Ident {
        format_ident!("get_{}", self.ident.as_ref().unwrap())
    }

    fn set_fn_ident(&self) -> Ident {
        format_ident!("set_{}", self.ident.as_ref().unwrap())
    }

    fn field_max(&self) -> u32 {
        if self.signed {
            2u32.pow(self.field_size() as u32 - 1) - 1
        } else {
            2u32.pow(self.field_size() as u32) - 1
        }
    }

    fn field_min(&self) -> i32 {
        if self.signed {
            !self.field_max() as i32
        } else {
            0
        }
    }

    fn field_size(&self) -> usize {
        self.msb - self.lsb + 1
    }

    fn field_mask(&self) -> u32 {
        (2u32.pow(self.field_size() as u32) - 1) << self.lsb
    }

    fn io_ty(&self, size: usize) -> Type {
        match size {
            8 => {
                if self.signed {
                    parse_quote!(i8)
                } else {
                    parse_quote!(u8)
                }
            }
            16 => {
                if self.signed {
                    parse_quote!(i16)
                } else {
                    parse_quote!(u16)
                }
            }
            32 => {
                if self.signed {
                    parse_quote!(i32)
                } else {
                    parse_quote!(u32)
                }
            }
            64 => {
                if self.signed {
                    parse_quote!(i64)
                } else {
                    parse_quote!(u64)
                }
            }
            _ => panic!("Invalid register size"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_max() {
        let field = Field {
            msb: 3,
            lsb: 0,
            signed: false,
            ..Default::default()
        };
        assert_eq!(field.field_max(), 15);

        let field = Field {
            msb: 16,
            lsb: 10,
            signed: true,
            ..Default::default()
        };
        assert_eq!(field.field_max(), 63);
    }

    #[test]
    fn test_field_min() {
        let field = Field {
            msb: 3,
            lsb: 0,
            signed: false,
            ..Default::default()
        };
        assert_eq!(field.field_min(), 0);

        let field = Field {
            msb: 16,
            lsb: 10,
            signed: true,
            ..Default::default()
        };
        assert_eq!(field.field_min(), -64);
    }

    #[test]
    fn test_field_size() {
        let field = Field {
            msb: 3,
            lsb: 0,
            signed: false,
            ..Default::default()
        };
        assert_eq!(field.field_size(), 4);

        let field = Field {
            msb: 16,
            lsb: 10,
            signed: true,
            ..Default::default()
        };
        assert_eq!(field.field_size(), 7);
    }

    #[test]
    fn test_field_mask() {
        let field = Field {
            msb: 3,
            lsb: 0,
            signed: false,
            ..Default::default()
        };
        assert_eq!(field.field_mask(), 0b1111);

        let field = Field {
            msb: 16,
            lsb: 10,
            signed: true,
            ..Default::default()
        };
        assert_eq!(field.field_mask(), 0b1_1111_1100_0000_0000);
    }

    #[test]
    fn test_signed_abs_field_mask() {
        let field = Field {
            msb: 3,
            lsb: 0,
            signed: true,
            ..Default::default()
        };
        assert_eq!(field.signed_field_mask_nosignbit(), 0b0111);

        let field = Field {
            msb: 16,
            lsb: 10,
            signed: true,
            ..Default::default()
        };
        assert_eq!(field.signed_field_mask_nosignbit(), 0b0_1111_1100_0000_0000);
    }
}
