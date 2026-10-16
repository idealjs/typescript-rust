use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_for_string_literal8() {
    let content = r#"// @stableTypeOrdering: true
type As = 'arf' | 'abacus' | 'abaddon';
let a: As;
if (a === '/**/"#;
    let mut s = Session::new_for_test("completionForStringLiteral8", content);
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["abacus", "abaddon", "arf"]);
}
