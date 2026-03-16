use darling::{FromMeta, ast::NestedMeta};
use syn::{ItemStruct, parse_macro_input};

use crate::register::Register;

mod field;
mod register;
mod util;

const SUPPORTED_WIDTHS: [usize; 5] = [8usize, 16usize, 32usize, 64usize, 128usize];

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

/// Generate register handling code for a register definition.
///
/// This macro may only be used on structs, and supports only registers with the widths of 8 bits,
/// 16 bits, 32 bits, 64 bits, or 128 bits.
///
/// This size *must* be declared as an attribute parameter, like so:
/// ```ignore
/// #[register(size = 16)]
/// struct MyRegister {}
/// ```
///
/// Fields may be declared in a similar syntax to the fields of a normal struct. They must define,
/// as attributes, the least significant bit (lsb) and most significant bit (msb). They must also
/// specify their sign via the "type" of the field. This type must either be "u" for unsigned or
/// "i" for signed. They may also optionally specify whether or not that field may be read or
/// written to, defaulting to true.
///
/// ```ignore
/// #[register(size = 16)]
/// struct MyRegister {
///     // Readable, writable, unsigned
///     #[field(lsb = 0, msb = 7)]
///     lower: u,
///
///     // Readable, signed
///     #[field(lsb = 8, msb = 15, write = false)]
///     upper: i,
/// }
///
/// let mut reg = MyRegister::new(0x04AD);
/// assert_eq!(reg.get_lower(), 0xAD);
/// assert_eq!(reg.get_upper(), 4);
/// assert!(reg.set_lower(-4).is_ok());
/// assert_eq!(reg.get_lower(), -4);
/// ```
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
