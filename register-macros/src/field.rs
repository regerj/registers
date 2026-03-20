use darling::FromField;
use quote::{ToTokens, format_ident};
use syn::{Ident, ItemFn, Type, parse_quote};

use crate::util::{sign, unsign};

#[derive(Debug)]
enum Number {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
}

impl PartialEq<u128> for Number {
    fn eq(&self, other: &u128) -> bool {
        match self {
            Number::U8(n) => *n as u128 == *other,
            Number::U16(n) => *n as u128 == *other,
            Number::U32(n) => *n as u128 == *other,
            Number::U64(n) => *n as u128 == *other,
            Number::U128(n) => *n == *other,
            Number::I8(n) => {
                if *n < 0 {
                    false
                } else {
                    *n as u128 == *other
                }
            }
            Number::I16(n) => {
                if *n < 0 {
                    false
                } else {
                    *n as u128 == *other
                }
            }
            Number::I32(n) => {
                if *n < 0 {
                    false
                } else {
                    *n as u128 == *other
                }
            }
            Number::I64(n) => {
                if *n < 0 {
                    false
                } else {
                    *n as u128 == *other
                }
            }
            Number::I128(n) => {
                if *n < 0 {
                    false
                } else {
                    *n as u128 == *other
                }
            }
        }
    }
}

impl PartialEq<i128> for Number {
    fn eq(&self, other: &i128) -> bool {
        match self {
            Number::U8(n) => *n as i128 == *other,
            Number::U16(n) => *n as i128 == *other,
            Number::U32(n) => *n as i128 == *other,
            Number::U64(n) => *n as i128 == *other,
            Number::U128(n) => {
                if *n > i128::MAX as u128 {
                    false
                } else {
                    *n as i128 == *other
                }
            }
            Number::I8(n) => *n as i128 == *other,
            Number::I16(n) => *n as i128 == *other,
            Number::I32(n) => *n as i128 == *other,
            Number::I64(n) => *n as i128 == *other,
            Number::I128(n) => *n == *other,
        }
    }
}

impl ToTokens for Number {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let lit: syn::LitInt = match self {
            Number::U8(n) => parse_quote!(#n),
            Number::U16(n) => parse_quote!(#n),
            Number::U32(n) => parse_quote!(#n),
            Number::U64(n) => parse_quote!(#n),
            Number::U128(n) => parse_quote!(#n),
            Number::I8(n) => parse_quote!(#n),
            Number::I16(n) => parse_quote!(#n),
            Number::I32(n) => parse_quote!(#n),
            Number::I64(n) => parse_quote!(#n),
            Number::I128(n) => parse_quote!(#n),
        };
        let lit = lit.base10_digits();
        let clean = syn::parse_str::<syn::LitInt>(lit).unwrap();
        clean.to_tokens(tokens);
    }
}

#[derive(Default, PartialEq)]
pub enum Typ {
    #[default]
    Unsigned,
    Signed,
    Flag,
}

#[derive(FromField)]
#[darling(attributes(field), and_then = finish)]
pub struct Field {
    pub ident: Option<syn::Ident>,
    ty: syn::Type,
    #[darling(skip)]
    pub typ: Typ,
    pub msb: usize,
    pub lsb: usize,
    #[darling(default = || true)]
    pub write: bool,
    #[darling(default = || true)]
    pub read: bool,
}

fn finish(mut field: Field) -> darling::Result<Field> {
    let error = Err(
        darling::Error::unexpected_type(&field.ty.to_token_stream().to_string())
            .with_span(&field.ty),
    );
    let typ = match &field.ty {
        Type::Path(path) => {
            if path.path.is_ident("u") {
                Typ::Unsigned
            } else if path.path.is_ident("i") {
                Typ::Signed
            } else if path.path.is_ident("b") {
                Typ::Flag
            } else {
                return error;
            }
        }
        _ => return error,
    };

    field.typ = typ;

    Ok(field)
}

impl Default for Field {
    fn default() -> Self {
        Self {
            ident: None,
            ty: parse_quote!(u),
            typ: Typ::default(),
            msb: 15,
            lsb: 0,
            write: true,
            read: true,
        }
    }
}

impl Field {
    pub fn get_impl(&self, reg_size: usize) -> ItemFn {
        match self.typ {
            Typ::Signed => self.signed_get_impl(reg_size),
            Typ::Unsigned => self.unsigned_get_impl(reg_size),
            Typ::Flag => unimplemented!()
        }
    }

    fn signed_get_impl(&self, reg_size: usize) -> ItemFn {
        assert!(matches!(self.typ, Typ::Signed));
        let fn_ident = self.get_fn_ident();
        let lsb = self.lsb;
        let msb = self.msb;
        let abs_field_mask = self.signed_field_mask_nosignbit(reg_size);
        let ty = self.io_ty(reg_size);

        parse_quote! {
            pub fn #fn_ident(&self) -> #ty {
                let signed = (self.reg >> #msb) & 1;
                if signed == 1 {
                    (!(#abs_field_mask >> #lsb) | (self.reg & #abs_field_mask) >> #lsb) as #ty
                } else {
                    ((self.reg & #abs_field_mask) >> #lsb) as #ty
                }
            }
        }
    }

    fn unsigned_get_impl(&self, reg_size: usize) -> ItemFn {
        assert!(matches!(self.typ, Typ::Unsigned));
        let fn_ident = self.get_fn_ident();
        let ty = self.io_ty(reg_size);
        let lsb = self.lsb;
        let field_mask = self.field_mask(reg_size);
        parse_quote! {
            pub fn #fn_ident(&self) -> #ty {
                (self.reg & #field_mask) >> #lsb
            }
        }
    }

    pub fn set_impl(&self, reg_size: usize) -> ItemFn {
        match self.typ {
            Typ::Signed => self.signed_set_impl(reg_size),
            Typ::Unsigned => self.unsigned_set_impl(reg_size),
            Typ::Flag => unimplemented!()
        }
    }

    fn signed_field_mask_nosignbit(&self, reg_size: usize) -> Number {
        assert!(matches!(self.typ, Typ::Signed));
        match self.field_mask(reg_size) {
            Number::U8(n) => Number::U8(n & !(1 << self.msb)),
            Number::U16(n) => Number::U16(n & !(1 << self.msb)),
            Number::U32(n) => Number::U32(n & !(1 << self.msb)),
            Number::U64(n) => Number::U64(n & !(1 << self.msb)),
            Number::U128(n) => Number::U128(n & !(1 << self.msb)),
            _ => unreachable!(),
        }
    }

    fn unsigned_set_impl(&self, reg_size: usize) -> ItemFn {
        assert!(matches!(self.typ, Typ::Unsigned));
        let ty = self.io_ty(reg_size);
        let fn_ident = self.set_fn_ident();
        let field_max = self.field_max(reg_size);
        let field_mask = self.field_mask(reg_size);
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

    fn signed_set_impl(&self, reg_size: usize) -> ItemFn {
        assert!(matches!(self.typ, Typ::Signed));
        let ty = self.io_ty(reg_size);
        let fn_ident = self.set_fn_ident();
        let field_mask = self.field_mask(reg_size);
        let abs_field_mask = self.signed_field_mask_nosignbit(reg_size);
        let msb = self.msb;
        let lsb = self.lsb;
        let field_max = self.field_max(reg_size);
        let field_min = self.field_min(reg_size);
        let sign_type = sign(reg_size);
        let unsign_type = unsign(reg_size);

        parse_quote! {
            pub fn #fn_ident(&mut self, val: #ty) -> registers::Result<()> {
                if val > #field_max as #sign_type || val < #field_min {
                    return Err(registers::Error::OutOfBoundsFieldWrite);
                }

                let signed_bit = if val < 0 {
                    1 << #msb
                } else {
                    0
                };

                self.reg = self.reg & !(#field_mask);
                self.reg = self.reg | signed_bit | ((val as #unsign_type & (#abs_field_mask >> #lsb)) << #lsb);

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

    fn field_max(&self, reg_size: usize) -> Number {
        let offset = match self.typ {
            Typ::Unsigned => 0,
            Typ::Signed => 1,
            Typ::Flag => unimplemented!(),
        };
        match reg_size {
            8 => Number::U8(2u8.pow(self.field_size() as u32 - offset) - 1),
            16 => Number::U16(2u16.pow(self.field_size() as u32 - offset) - 1),
            32 => Number::U32(2u32.pow(self.field_size() as u32 - offset) - 1),
            64 => Number::U64(2u64.pow(self.field_size() as u32 - offset) - 1),
            128 => Number::U128(2u128.pow(self.field_size() as u32 - offset) - 1),
            _ => unreachable!(),
        }
    }

    fn field_min(&self, reg_size: usize) -> Number {
        assert!(matches!(self.typ, Typ::Signed | Typ::Unsigned));
        if self.typ == Typ::Unsigned {
            return Number::U8(0);
        }

        match self.field_max(reg_size) {
            Number::U8(n) => Number::I8(!n as i8),
            Number::U16(n) => Number::I16(!n as i16),
            Number::U32(n) => Number::I32(!n as i32),
            Number::U64(n) => Number::I64(!n as i64),
            Number::U128(n) => Number::I128(!n as i128),
            _ => unreachable!(),
        }
    }

    fn field_size(&self) -> usize {
        self.msb - self.lsb + 1
    }

    fn field_mask(&self, reg_size: usize) -> Number {
        match reg_size {
            8 => Number::U8((2u8.pow(self.field_size() as u32) - 1) << self.lsb),
            16 => Number::U16((2u16.pow(self.field_size() as u32) - 1) << self.lsb),
            32 => Number::U32((2u32.pow(self.field_size() as u32) - 1) << self.lsb),
            64 => Number::U64((2u64.pow(self.field_size() as u32) - 1) << self.lsb),
            128 => Number::U128((2u128.pow(self.field_size() as u32) - 1) << self.lsb),
            _ => unreachable!(),
        }
    }

    fn io_ty(&self, reg_size: usize) -> Type {
        if self.typ == Typ::Flag {
            return parse_quote!(bool);
        }

        match reg_size {
            8 => {
                match self.typ {
                    Typ::Signed => parse_quote!(i8),
                    Typ::Unsigned => parse_quote!(u8),
                    _ => unreachable!(),
                }
            }
            16 => {
                match self.typ {
                    Typ::Signed => parse_quote!(i16),
                    Typ::Unsigned => parse_quote!(u16),
                    _ => unreachable!(),
                }
            }
            32 => {
                match self.typ {
                    Typ::Signed => parse_quote!(i32),
                    Typ::Unsigned => parse_quote!(u32),
                    _ => unreachable!(),
                }
            }
            64 => {
                match self.typ {
                    Typ::Signed => parse_quote!(i64),
                    Typ::Unsigned => parse_quote!(u64),
                    _ => unreachable!(),
                }
            }
            128 => {
                match self.typ {
                    Typ::Signed => parse_quote!(i128),
                    Typ::Unsigned => parse_quote!(u128),
                    _ => unreachable!(),
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
            typ: Typ::Unsigned,
            ..Default::default()
        };
        assert_eq!(field.field_max(32), 15u128);

        let field = Field {
            msb: 16,
            lsb: 10,
            typ: Typ::Signed,
            ..Default::default()
        };
        assert_eq!(field.field_max(32), 63u128);
    }

    #[test]
    fn test_field_min() {
        let field = Field {
            msb: 3,
            lsb: 0,
            typ: Typ::Unsigned,
            ..Default::default()
        };
        assert_eq!(field.field_min(32), 0u128);

        let field = Field {
            msb: 16,
            lsb: 10,
            typ: Typ::Signed,
            ..Default::default()
        };
        assert_eq!(field.field_min(32), -64i128);
    }

    #[test]
    fn test_field_size() {
        let field = Field {
            msb: 3,
            lsb: 0,
            typ: Typ::Unsigned,
            ..Default::default()
        };
        assert_eq!(field.field_size(), 4);

        let field = Field {
            msb: 16,
            lsb: 10,
            typ: Typ::Signed,
            ..Default::default()
        };
        assert_eq!(field.field_size(), 7);
    }

    #[test]
    fn test_field_mask() {
        let field = Field {
            msb: 3,
            lsb: 0,
            typ: Typ::Unsigned,
            ..Default::default()
        };
        assert_eq!(field.field_mask(32), 0b1111u128);

        let field = Field {
            msb: 16,
            lsb: 10,
            typ: Typ::Signed,
            ..Default::default()
        };
        assert_eq!(field.field_mask(32), 0b1_1111_1100_0000_0000u128);

        let field = Field {
            msb: 15,
            lsb: 1,
            typ: Typ::Signed,
            ..Default::default()
        };
        assert!(matches!(
            field.field_mask(16),
            Number::U16(0b1111_1111_1111_1110)
        ));
    }

    #[test]
    fn test_signed_abs_field_mask() {
        let field = Field {
            msb: 3,
            lsb: 0,
            typ: Typ::Signed,
            ..Default::default()
        };
        assert_eq!(field.signed_field_mask_nosignbit(32), 0b0111u128);

        let field = Field {
            msb: 16,
            lsb: 10,
            typ: Typ::Signed,
            ..Default::default()
        };
        assert_eq!(
            field.signed_field_mask_nosignbit(32),
            0b0_1111_1100_0000_0000u128
        );

        let field = Field {
            msb: 15,
            lsb: 1,
            typ: Typ::Signed,
            ..Default::default()
        };
        assert!(matches!(
            field.signed_field_mask_nosignbit(16),
            Number::U16(0b0111_1111_1111_1110)
        ));
    }

    #[test]
    fn test_from_field() {
        let field: syn::Field = parse_quote! {
            #[field(lsb = 0, msb = 15)]
            foo: i
        };

        let field = Field::from_field(&field).expect("Failed to parse");
        assert_eq!(field.ident.map(|i| i.to_string()), Some("foo".to_string()));
        assert_eq!(field.lsb, 0);
        assert_eq!(field.msb, 15);
        assert!(field.typ == Typ::Signed);

        let field: syn::Field = parse_quote! {
            #[field(lsb = 0, msb = 15)]
            foo: i32
        };

        assert!(Field::from_field(&field).is_err());
    }
}
