use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn completion_for_string_literal4() {
    let content = r#"// @allowJs: true
// @Filename: in.js
/** I am documentation
 * @param {'literal'} p1
 * @param {"literal"} p2
 * @param {'other1' | 'other2'} p3
 * @param {'literal' | number} p4
 * @param {12 | true} p5
 */
function f(p1, p2, p3, p4, p5) {
    return p1 + p2 + p3 + p4 + p5 + '.';
}
f/*1*/('literal', 'literal', "[|o/*2*/ther1|]", 12);"#;
    let mut s = Session::new_for_test("completionForStringLiteral4", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyQuickInfoExists(t)
    // TODO: f.VerifyQuickInfoIs(t, "function f(p1: \"literal\", p2: \"literal\", p3: \"other1\" | \"other2\", p4
    fourslash::go_to_marker(&mut s, "2");
    // TODO: f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
}
