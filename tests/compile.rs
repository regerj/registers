#[test]
fn invalid_register_size() {
    let t = trybuild::TestCases::new();
    t.compile_fail("compile-tests/*.rs");
}
