use darling::FromField;
use syn::parse_quote;

use crate::common::Field;

pub struct Register32 {
    ident: syn::Ident,
    fields: Vec<Field>,
}

impl Register32 {
    pub fn derive_from_u32() -> syn::ItemImpl {
        parse_quote! {
            impl From<u32> for
        }
    }
}

//impl From<&syn::DataStruct> for Register32 {
//    fn from(value: &syn::DataStruct) -> Self {
//        let fields: Vec<_> = value.fields.iter().map(|f| Field::from_field(f).unwrap()).collect();
//
//        Register32 { fields }
//    }
//}
