use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_import_clause05() {
    let content = r#"// @Filename: app.ts
import * as A from "/*1*/";
// @Filename: /node_modules/@types/a__b/index.d.ts
declare module "@e/f" { function fun(): string; }
// @Filename: /node_modules/@types/c__d/index.d.ts
export declare let x: number;"#;
    let mut s = Session::new_for_test("completionListInImportClause05", content);
    fourslash::verify_completions_unsorted_at(&mut s, Some("1"), &["@e/f", "@a/b", "@c/d"]);
}
