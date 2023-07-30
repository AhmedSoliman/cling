#[cfg(feature = "derive")]
#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.pass("tests/derive/*-success.rs");
    t.compile_fail("tests/derive/*-fail.rs");
}
