use tsox_lsp::fourslash::Session;


#[test]
fn quickinfo_verbosity1() {
    let content = r#"type FooType = string | number;
const foo/*a*/: FooType = 1;
type BarType = FooType | boolean;
const bar/*b*/: BarType = 1;"#;
    let _s = Session::new_for_test("quickinfoVerbosity1", content);
    // TODO: f.VerifyBaselineHoverWithVerbosity(t, map[string][]int{"a": {0, 1}, "b": {0, 1, 2}})
}
