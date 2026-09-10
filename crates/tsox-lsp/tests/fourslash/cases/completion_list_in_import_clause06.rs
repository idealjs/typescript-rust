use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_import_clause06() {
    let content = r#"// @typeRoots: T1,T2
// @Filename: app.ts
import * as A from "/*1*/";
// @Filename: T1/a__b/index.d.ts
export declare let x: number;
// @Filename: T2/a__b/index.d.ts
export declare let x: number;"#;
    let mut s = Session::new_for_test("completionListInImportClause06", content);
    fourslash::verify_completions_exact_at(&mut s, Some("1"), &["@a/b"]);
}
