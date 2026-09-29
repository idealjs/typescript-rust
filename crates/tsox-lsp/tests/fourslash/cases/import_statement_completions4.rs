use tsox_lsp::fourslash::Session;


#[test]
fn import_statement_completions4() {
    let content = r#"// @allowJs: true
// @Filename: /a.js
import Foo /*a*/

function fromBar() {}
// @Filename: /b.jsx
import Foo /*b*/

function fromBar() {}"#;
    let _s = Session::new_for_test("importStatementCompletions4", content);
    // TODO: f.VerifyCompletions(t, []string{"a", "b"}, &fourslash.CompletionsExpectedList{
}
