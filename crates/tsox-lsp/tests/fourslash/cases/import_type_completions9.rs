use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn import_type_completions9() {
    let content = r#"// @target: esnext
// @filename: /foo.ts
export interface Foo {}
// @filename: /bar.ts
[|import { type /**/ }|]"#;
    let mut s = Session::new(content);
    fourslash::go_to_file(&mut s, "/bar.ts");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
