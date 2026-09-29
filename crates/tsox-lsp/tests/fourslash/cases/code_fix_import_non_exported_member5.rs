use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_import_non_exported_member5() {
    let content = r#"// @moduleResolution: bundler
// @module: esnext
// @filename: /node_modules/foo/index.js
function bar() {}
// @filename: /b.ts
import { bar } from "./foo";"#;
    let mut s = Session::new_for_test("codeFixImportNonExportedMember5", content);
    fourslash::go_to_file(&mut s, "/b.ts");
    // TODO: f.VerifyCodeFixNotAvailable(t, "fixImportNonExportedMember")
}
