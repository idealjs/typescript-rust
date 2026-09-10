use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyImportFixAtPosition"]
#[test]
fn import_name_code_fix_dollar() {
    let content = r#"// @module: esnext
// @moduleResolution: bundler
// @Filename: /node_modules/qwik/index.d.ts
export declare const $: any;
// @Filename: /index.ts
import {} from "qwik";
$/**/"#;
    let mut s = Session::new_for_test("importNameCodeFix_dollar", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}
