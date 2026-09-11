use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_in_js_doc_qualified_names() {
    let content = r#"// @allowJs: true
// @Filename: /node_modules/foo/index.d.ts
/** tee */
export type T = number;
// @Filename: /a.js
import * as Foo from "foo";
/** @type {Foo./**/} */
const x = 0;"#;
    let mut s = Session::new_for_test("completionInJsDocQualifiedNames", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
