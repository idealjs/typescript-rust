use tsox_lsp::fourslash::{self, Session};


#[test]
fn auto_import_typedef_missing_name() {
    let content = r#"// @allowJs: true
// @checkJs: true
// @Filename: /utils.js
/** @typedef {{ x: number }} */

export function doSomething() {}
// @Filename: /index.ts
doSomething/**/"#;
    let mut s = Session::new_for_test("autoImportTypedefMissingName", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.BaselineAutoImportsCompletions(t, []string{""})
}
