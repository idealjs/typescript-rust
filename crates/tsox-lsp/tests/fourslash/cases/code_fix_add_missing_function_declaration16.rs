use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_add_missing_function_declaration16() {
    let content = r#"// @moduleResolution: bundler
// @filename: /node_modules/test/index.js
export const x = 1;
// @filename: /foo.ts
import * as test from "test";
test.foo();"#;
    let mut s = Session::new_for_test("codeFixAddMissingFunctionDeclaration16", content);
    fourslash::go_to_file(&mut s, "/foo.ts");
    // TODO: f.VerifyCodeFixNotAvailable(t)
}
