use tsox_lsp::fourslash::{self, Session};


#[test]
fn quickinfo_verbosity_conditional_type() {
    let content = r#"interface Apple {
    color: string;
    weight: number;
}
type StrInt = string | bigint;
type T1<T extends Apple | Apple[]> = T extends { color: string } ? "one apple" : StrInt;
function f<T extends Apple | Apple[]>(x: T1<T>): void {
    x/*x*/;
}"#;
    let mut s = Session::new_for_test("quickinfoVerbosityConditionalType", content);
    // TODO: f.VerifyBaselineHoverWithVerbosity(t, map[string][]int{"x": {0, 1, 2}})
}
