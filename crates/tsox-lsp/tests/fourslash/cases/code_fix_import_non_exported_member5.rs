use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCodeFixNotAvailable"]
#[test]
fn code_fix_import_non_exported_member5() {
    let content = r#"// @moduleResolution: bundler
// @module: esnext
// @filename: /node_modules/foo/index.js
function bar() {}
// @filename: /b.ts
import { bar } from "./foo";"#;
    let mut s = Session::new(content);
    fourslash::go_to_file(&mut s, "/b.ts");
    fourslash::unsupported("VerifyCodeFixNotAvailable"); // f.VerifyCodeFixNotAvailable(t, "fixImportNonExportedMember")
}
