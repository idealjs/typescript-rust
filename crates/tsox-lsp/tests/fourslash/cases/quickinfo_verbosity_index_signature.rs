use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineHoverWithVerbosity"]
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
    let mut s = Session::new_for_test("quickinfoVerbosityIndexSignature", content);
    fourslash::unsupported("VerifyBaselineHoverWithVerbosity"); // f.VerifyBaselineHoverWithVerbosity(t, map[string][]int{"a": {0, 1}, "f": {0, 1, 2}})
}
