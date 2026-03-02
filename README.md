# Registers

> [!WARNING]
> This crate is a work in progress. Currently, only 32 bit registers are supported.

This crate is a simple procedural macro based crate for working with registers. The goal is to make supporting complex register fields as intuitive and safe as possible.

## Quickstart

Take for example the [First Common PCIe Header Register](https://wiki.osdev.org/PCI#Base_Address_Registers):

```rust
#[registers::register(size = 32, write = false)]
struct PCIeHeader0 {
    #[field(lsb = 0, msb = 15, write = false)]
    vendor_id: u,
    #[field(lsb = 16, msb = 31, write = false)]
    device_id: u,
}
```

The above declaration automatically generate implementations to work with the register in a safe manner:

```rust
const PCIE_HEADER_0_ADDR: *const u32 = 0xDEADBEEF as *const u32;

let mut reg = PCIeHeader0::new();
unsafe { reg.read(PCIE_HEADER_0_ADDR); }

println!("Vendor ID: {:#0>4X}", reg.get_vendor_id());
println!("Device ID: {:#0>4X}", reg.get_device_id());
```

Registers are RW by default, and either can be disabled by specifying `read = false` or `write = false`. This will toggle whether or not the `PCIeHeader0::read()` or `PCIeHeader0::write()` methods are implemented or not.

This same behaviour is supported per-field as well in the same manner, toggling if `get_*()` and `set_*()` methods are implemented.

Out of bounds writes are protected as well:

```rust
#[registers::register(size = 32)]
struct Foo {
    #[field(lsb = 0, msb = 3)]
    bar: u,
}

let foo = Foo::new();
assert!(foo.set_bar(0b1_0000).is_err());
```

There is no need to explicitly declare reserved bytes, though you can if you wish by marking both read and write as false.

```rust
#[registers::register(size = 32)]
struct Foo {
    #[field(lsb = 0, msb = 3, read = false, write = false)]
    reserved_0: u,
    #[field(lsb = 4, msb = 7)]
    bar: u,
    #[field(lsb = 8, msb = 31, read = false, write = false)]
    reserved_1: u,
}
```

Signed fields are supported (2s complement) by changing the type of the field to `i`:

```rust
#[registers::register(size = 32)]
struct Foo {
    #[field(lsb = 0, msb = 15)]
    bar: i,
}

let foo = Foo::new();
assert!(foo.set_bar(i16::MIN.into()).is_ok());
assert!(foo.get_bar(), i16::MIN.into());
```

This will cause the `get_*` and `set_*` to return and take in signed integer values, respectively.

## Overhead

The size of a register struct is guranteed to be no bigger than the underlying register itself, so the generated register struct for a 32 bit register is guranteed to be no larger than 32 bits.

## Alternatives
- [tock-registers](https://github.com/tock/tock-registers) - Another macro based register library

## License
This project is licensed under the [MIT License](./LICENSE.txt).
