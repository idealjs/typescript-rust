use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFixNotAvailable"]
#[test]
fn code_fix_import_non_exported_member4() {
    let content = r#"// @module: esnext
// @filename: /a.d.ts
declare function foo(): any;
declare function bar(): any;
// @filename: /b.ts
import { bar } from "./a";"#;
    let mut s = Session::new_for_test("codeFixImportNonExportedMember4", content);
    fourslash::go_to_file(&mut s, "/b.ts");
    fourslash::unsupported("VerifyCodeFixNotAvailable"); // f.VerifyCodeFixNotAvailable(t, "fixImportNonExportedMember")
}
