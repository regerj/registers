use registers::_register;

#[_register(size = 32)]
struct HIF {
    #[field(lsb: 0, msb: 15)]
    foo: u,
    #[field(lsb: 16, msb: 31)]
    bar: i,
}
