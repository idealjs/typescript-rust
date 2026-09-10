use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn code_fix_spelling_case_weight2() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"let ABCDEFGHI = 1;
let abcdefghij = 1;
[|abcdefghi|]"#;
    let mut s = Session::new_for_test("codeFixSpellingCaseWeight2", content);
    fourslash::unsupported("VerifyRangeAfterCodeFix"); // f.VerifyRangeAfterCodeFix(t, `ABCDEFGHI`, false, 0, 0)
}
