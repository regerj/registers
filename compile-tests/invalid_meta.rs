use registers::register;

#[register(size = 16, foo?)]
struct Foo {
    #[field(msb = 15, lsb = 0)]
    bar: u,
}

fn main() {
    todo!();
}
