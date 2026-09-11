use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_type_completions9() {
    let content = r#"// @target: esnext
// @filename: /foo.ts
export interface Foo {}
// @filename: /bar.ts
[|import { type /**/ }|]"#;
    let mut s = Session::new_for_test("importTypeCompletions9", content);
    fourslash::go_to_file(&mut s, "/bar.ts");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
