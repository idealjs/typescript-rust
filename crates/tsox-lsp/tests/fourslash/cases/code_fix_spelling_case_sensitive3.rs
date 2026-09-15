use tsox_lsp::fourslash::Session;


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn code_fix_spelling_case_sensitive3() {
    let content = r#"class Node {}
let node = new Node();
[|nodes|]"#;
    let _s = Session::new_for_test("codeFixSpellingCaseSensitive3", content);
    // TODO: f.VerifyRangeAfterCodeFix(t, `node`, false, 0, 0)
}
