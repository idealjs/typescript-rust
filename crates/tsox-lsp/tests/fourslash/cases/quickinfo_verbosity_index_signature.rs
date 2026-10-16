use tsox_lsp::fourslash::Session;


#[test]
fn quickinfo_verbosity_index_signature() {
    let content = r#"type Key = string | number;
interface Apple {
    banana: number;
}
interface Foo {
    [a/*a*/: Key]: Apple;
}
const f/*f*/: Foo = {};"#;
    let _s = Session::new_for_test("quickinfoVerbosityIndexSignature", content);
    // TODO: f.VerifyBaselineHoverWithVerbosity(t, map[string][]int{"a": {0, 1}, "f": {0, 1, 2}})
}
