#![no_std]

//! # Description
//! This library provides a convenient attribute macro for working with segmented registers in
//! Rust.
//!
//! Registers often have multiple distinct fields of varying widths and varying types. The typical
//! way to assign and retrieve the values in these fields have been with bit shifting operations.
//! This logic is error prone, and the code surface area that needs to be carefully examined is not
//! insignificant.
//!
//! The macro provided by this library generates this bit shifting code for you, wrapping it up
//! into an idiomatic struct pattern that Rust users will be familiar with.
//!
//! # Features
//!
//! - Custom bit precise field sizes
//! - 2s complement signed fields
//! - Support for 8, 16, 32, 64, and 128 bit registers
//! - Toggleable MMIO read and write methods
//! - Toggleable field get and set methods
//! - Zero cost abstraction
//!
//! # Example
//!
//! For example, take a PCI memory space BAR:
//!
//! | Bits 31-4    | Bit 3        | Bits 2-1 | Bit 0    |
//! | ------------ | ------------ | -------- | -------- |
//! | Base Address | Prefetchable | Type     | Always 0 |
//!
//! This register could be supported like so:
//!
//! ```
//! # use registers::register;
//! #[register(size = 32, write = false)]
//! struct PCI_BAR {
//!     #[field(lsb = 4, msb = 31, write = false)]
//!     base_addr: u,
//!     #[field(lsb = 3, msb = 3, write = false)]
//!     prefetch: u,
//!     #[field(lsb = 1, msb = 2, write = false)]
//!     ty: u,
//!     #[field(lsb = 0, msb = 0, write = false)]
//!     reserved: u,
//! }
//!
//! //                  Address      Meta
//! let mock_bar: u32 = 0xDEADBEE0 + 0b1100;
//! let mut bar = PCI_BAR::new();
//! let bar_addr = &mock_bar as *const u32;
//! unsafe { bar.read(bar_addr) }
//!
//! assert_eq!(bar.get_base_addr(), 0xDEADBEE);
//! assert_eq!(bar.get_prefetch(), 1);
//! assert_eq!(bar.get_ty(), 0b10);
//! assert_eq!(bar.get_reserved(), 0);
//! ```
//!
//! ## Note
//!
//! Reserved bits are not *required* to be specified. They may be omitted if desired.
//!
//! For a control register (writable) let's make up a register definition:
//!
//! | Bits 15-1            | Bit 0  |
//! | -------------------- | ------ |
//! | Coefficient (signed) | Enable |
//!
//! Support for this register might look like:
//!
//! ```
//! # use registers::register;
//! #[register(size = 16, read = false)]
//! struct CSR {
//!     #[field(msb = 15, lsb = 1)]
//!     coeff: i,
//!     #[field(msb = 0, lsb = 0)]
//!     enable: u,
//! }
//!
//! # fn main() -> registers::Result<()> {
//! let mut csr = CSR::new();
//! csr.set_coeff(-14)?;
//! csr.set_enable(1)?;
//!
//! // Set will also bit bounds check
//! assert!(csr.set_enable(0b11).is_err());
//!
//! assert_eq!(csr.get_coeff(), -14);
//! assert_eq!(csr.get_enable(), 1);
//! # Ok(())
//! # }
//! ```
//!
//! # Knobs
//!
//! Fields must be specified as one of the following types:
//!
//! | Type | Description            |
//! | ---- | ---------------------- |
//! | u    | Unsigned               |
//! | i    | Signed (2s complement) |
//!
//! The `register` attribute supports the following metadata:
//!
//! | Name  | Required | Description |
//! | ----- | -------- | ----------- |
//! | size  | True     | Size of register in bits |
//! | read  | False    | Whether this register is readable from memory, controls the .read() method, defaults to true |
//! | write | False    | Whether this register is writeable from memory, controls the .write()
//! method, defaults to true |
//!
//! Each `field` attribute supports the following metadata:
//!
//! | Name  | Required | Description |
//! | ----- | -------- | ----------- |
//! | lsb   | True     | Index of the least significant bit |
//! | msb   | True     | Index of the most significant bit |
//! | read  | False    | Whether this field should be readable, defaults to true |
//! | write | False    | Whether this field should be writable, defaults to true |
//!
//! # Overhead
//!
//! ## Size
//!
//! The actual struct representation contains just a single internal unsigned integer of the same
//! size as the register. This means that an instance of this struct is the same size as the u32
//! you would have used to hold the value from that memory address.
//!
//! ## Performance
//!
//! The compiler will inline the implementations, meaning that the bitwise shifting and masking
//! will compile to essentially the same machine instructions as a manual implementation would
//! have.
pub use register_macros::register;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum Error {
    #[error("Field write out of bounds")]
    OutOfBoundsFieldWrite,
}

pub type Result<T> = core::result::Result<T, Error>;
