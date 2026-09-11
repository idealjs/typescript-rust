use tsox_lsp::fourslash::{self, Session};


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
    let mut s = Session::new_for_test("completionImportMeta", content);
    fourslash::verify_completions_exact_at(&mut s, Some("1"), &["meta"]);
    fourslash::verify_completions_empty_at(&mut s, Some("2"));
    fourslash::verify_completions_exact_at(&mut s, Some("3"), &["meta"]);
}
