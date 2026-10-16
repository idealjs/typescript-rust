use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_name_code_fix_sort_by_distance() {
    let content = r#"// @module: commonjs
// @Filename: /src/admin/utils/db/db.ts
export const db = {};
// @Filename: /src/admin/utils/db/index.ts
export * from "./db";
// @Filename: /src/client/helpers/db.ts
export const db = {};
// @Filename: /src/client/db.ts
export const db = {};
// @Filename: /src/client/foo.ts
db/**/"#;
    let mut s = Session::new_for_test("importNameCodeFix_sortByDistance", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}
