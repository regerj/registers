use registers::register;

#[register(size = 7)]
struct Foo {
    #[field(msb = 7, lsb = 0)]
    bar: u,
}

fn main() {
    todo!();
}
