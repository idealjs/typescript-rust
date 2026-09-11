use tsox_lsp::fourslash::{self, Session};


#[test]
fn javascript_modules23() {
    let content = r#"// @Filename: mod.ts
var foo = {a: "test"};
export = foo;
// @Filename: app.ts
import {a} from "./mod"
a./**/"#;
    let mut s = Session::new_for_test("javascriptModules23", content);
    fourslash::verify_completions_include_exclude_at(&mut s, Some(""), &["toString"], &[]);
}
