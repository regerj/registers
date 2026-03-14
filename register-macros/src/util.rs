pub fn sign(size: usize) -> syn::Type {
    syn::parse_str(&format!("i{}", size)).unwrap()
}

pub fn unsign(size: usize) -> syn::Type {
    syn::parse_str(&format!("u{}", size)).unwrap()
}
