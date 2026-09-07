use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.BaselineAutoImportsCompletions"]
#[test]
fn auto_import_typedef_missing_name() {
    let content = r#"// @allowJs: true
// @checkJs: true
// @Filename: /utils.js
/** @typedef {{ x: number }} */

export function doSomething() {}
// @Filename: /index.ts
doSomething/**/"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("BaselineAutoImportsCompletions"); // f.BaselineAutoImportsCompletions(t, []string{""})
}
