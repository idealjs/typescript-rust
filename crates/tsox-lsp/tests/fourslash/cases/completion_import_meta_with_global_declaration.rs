use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn completion_import_meta_with_global_declaration() {
    let content = r#"// @Filename: a.ts
import./*1*/
// @Filename: b.ts
declare global {
  interface ImportMeta {
    url: string;
  }
}
import.meta./*2*/
// @Filename: c.ts
import.meta./*3*/url
// @Filename: d.ts
import./*4*/meta"#;
    let mut s = Session::new_for_test("completionImportMetaWithGlobalDeclaration", content);
    fourslash::verify_completions_exact_at(&mut s, Some("1"), &["meta"]);
    fourslash::go_to_marker(&mut s, "2");
    // TODO: f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "3");
    // TODO: f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
    fourslash::verify_completions_exact_at(&mut s, Some("4"), &["meta"]);
}
