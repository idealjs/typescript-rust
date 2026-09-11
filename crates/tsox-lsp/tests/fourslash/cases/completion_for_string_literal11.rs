use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_for_string_literal11() {
    let content = r#"// @stableTypeOrdering: true
type As = 'arf' | 'abacus' | 'abaddon';
let a: As;
switch (a) {
    case '[|/**/|]
}"#;
    let mut s = Session::new_for_test("completionForStringLiteral11", content);
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["abacus", "abaddon", "arf"]);
}
