use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_non_existent_import() {
    let content = r#"import { NonExistentType } from "non-existent-module";
let foo: /**/"#;
    let mut s = Session::new_for_test("completionsNonExistentImport", content);
    fourslash::verify_completions_include_exclude_at(&mut s, Some(""), &["NonExistentType"], &[]);
}
