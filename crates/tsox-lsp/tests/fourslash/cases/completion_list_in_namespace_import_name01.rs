use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_namespace_import_name01() {
    let content = r#"// @Filename: m1.ts
export var foo: number = 1;
// @Filename: m2.ts
import * as /**/ from "m1""#;
    let mut s = Session::new_for_test("completionListInNamespaceImportName01", content);
    fourslash::verify_completions_empty_at(&mut s, Some(""));
}
