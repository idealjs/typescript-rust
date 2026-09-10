use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineSelectionRanges"]
#[test]
fn smart_selection_complex() {
    let content = r#"type X<T, P> = IsExactlyAny<P> extends true ? T : ({ [K in keyof P]: IsExactlyAny<P[K]> extends true ? K extends keyof T ? T[K] : P[/**/K] : P[K]; } & Pick<T, Exclude<keyof T, keyof P>>)"#;
    let mut s = Session::new_for_test("smartSelection_complex", content);
    fourslash::unsupported("VerifyBaselineSelectionRanges"); // f.VerifyBaselineSelectionRanges(t)
}
