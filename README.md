# Registers

This crate is a simple procedural macro based crate for working with registers. The goal is to make supporting complex register fields as intuitive and safe as possible.

## Quickstart

Take for example the [First Common PCIe Header Register](https://wiki.osdev.org/PCI#Base_Address_Registers):

```rust
#[registers::register(write = false)]
struct PCIeHeader0 {
    #[field(lsb = 0, msb = 15, write = false)]
    vendor_id: u32,
    #[field(lsb = 16, msb = 31, write = false)]
    device_id: u32,
}
```

The above declaration automatically generate implementations to work with the register in a safe manner:

```rust
const reg_addr = 0xDEADBEEF as *const u32;

assert_eq!(size_of::<PCIeHeader0>(), size_of::<u32>());

let mut reg = PCIeHeader0::new();
unsafe { reg.read(reg_addr); }

println!("Vendor ID: {:#0>4X}", reg.get_vendor_id());
println!("Device ID: {:#0>4X}", reg.get_device_id());
```

Registers are RW by default, and either can be disabled by specifying `read = false` or `write = false`. This will toggle whether or not the `PCIeHeader0::read()` or `PCIeHeader0::write()` methods are implemented or not.

This same behaviour is supported per-field as well in the same manner.

Out of bounds writes are protected as well:

```rust
#[registers::register]
struct Foo {
    #[field(lsb = 0, msb = 3)]
    bar: u32,
}

let foo = Foo::new();
assert!(foo.set_bar(0b1_0000).is_err());
```

There is no need to explicitly declare reserved bytes, though you can if you wish by marking both read and write as false.

```rust
#[registers::register]
struct Foo {
    #[field(lsb = 0, msb = 3, read = false, write = false)]
    reserved_0: u32,
    #[field(lsb = 4, msb = 7)]
    bar: u32,
    #[field(lsb = 8, msb = 31, read = false, write = false)]
    reserved_1: u32,
}
```

Signed fields are supported (2s complement):

```rust
#[registers::register]
struct Foo {
    #[field(lsb = 0, msb = 15, signed = true)]
    bar: u32,
}

let foo = Foo::new();
assert!(foo.set_bar(i16::MIN.into()).is_ok());
assert!(foo.get_bar(), i16::MIN.into());
```

## Alternatives
- [tock-registers](https://github.com/tock/tock-registers) - Another macro based register library

## License
This project is licensed under the [MIT License](./LICENSE.txt).
