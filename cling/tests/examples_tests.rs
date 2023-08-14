#[test]
fn trycmd() {
    let t = trycmd::TestCases::new();
    t.register_bins(trycmd::cargo::compile_examples([]).unwrap());
    t.case("../examples/cmd/*.toml");
}
