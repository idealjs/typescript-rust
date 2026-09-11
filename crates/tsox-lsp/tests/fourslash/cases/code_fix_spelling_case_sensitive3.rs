use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_spelling_case_sensitive3() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"class Node {}
let node = new Node();
[|nodes|]"#;
    let mut s = Session::new_for_test("codeFixSpellingCaseSensitive3", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `node`, false, 0, 0)
}
