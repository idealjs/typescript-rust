use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn code_fix_spelling_case_weight1() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"let ABCDEFGHIJKLMNOPQR = 1;
let abcdefghijklmnopqrs = 1;
[|abcdefghijklmnopqr|]"#;
    let mut s = Session::new_for_test("codeFixSpellingCaseWeight1", content);
    fourslash::unsupported("VerifyRangeAfterCodeFix"); // f.VerifyRangeAfterCodeFix(t, `abcdefghijklmnopqrs`, false, 0, 0)
}
