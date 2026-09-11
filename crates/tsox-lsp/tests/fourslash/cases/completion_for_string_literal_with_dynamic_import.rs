use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_for_string_literal_with_dynamic_import() {
    let content = r#"// @typeRoots: fourslash/my_typings
// @Filename: fourslash/test.ts
const a = import("./some/*0*/
const a = import("./sub/some/*1*/");
const a = import("[|some-/*2*/|]");
const a = import("..//*3*/");
// @Filename: fourslash/someFile1.ts
/*someFile1*/
// @Filename: fourslash/sub/someFile2.ts
/*someFile2*/
// @Filename: fourslash/my_typings/some-module/index.d.ts
export var x = 9;"#;
    let mut s = Session::new_for_test("completionForStringLiteralWithDynamicImport", content);
    fourslash::verify_completions_unsorted_at(&mut s, Some("0"), &["someFile1", "my_typings", "sub"]);
    fourslash::verify_completions_unsorted_at(&mut s, Some("1"), &["someFile2"]);
    // TODO: f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::verify_completions_unsorted_at(&mut s, Some("3"), &["fourslash"]);
}
