use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn completion_import_meta() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @lib: es5
// @Filename: a.ts
import./*1*/
// @Filename: b.ts
import.meta./*2*/
// @Filename: c.ts
import./*3*/meta"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", nil)
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
}
