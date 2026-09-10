use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn import_statement_completions4() {
    let content = r#"// @allowJs: true
// @Filename: /a.js
import Foo /*a*/

function fromBar() {}
// @Filename: /b.jsx
import Foo /*b*/

function fromBar() {}"#;
    let mut s = Session::new_for_test("importStatementCompletions4", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"a", "b"}, &fourslash.CompletionsExpectedList{
}
