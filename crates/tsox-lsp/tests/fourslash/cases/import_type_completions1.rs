use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_type_completions1() {
    let content = r#"// @target: esnext
// @filename: /foo.ts
export interface Foo {}
// @filename: /bar.ts
[|import type F/**/|]"#;
    let mut s = Session::new_for_test("importTypeCompletions1", content);
    fourslash::go_to_file(&mut s, "/bar.ts");
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
