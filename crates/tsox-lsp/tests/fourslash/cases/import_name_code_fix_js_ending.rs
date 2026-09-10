use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyImportFixModuleSpecifiers"]
#[test]
fn import_name_code_fix_js_ending() {
    let content = r#"// @lib: es5
// @module: commonjs
// @Filename: /node_modules/lit/package.json
{ "name": "lit", "version": "1.0.0" }
// @Filename: /node_modules/lit/index.d.ts
import "./decorators";
// @Filename: /node_modules/lit/decorators.d.ts
export declare function customElement(name: string): any;
// @Filename: /a.ts
customElement/**/"#;
    let mut s = Session::new_for_test("importNameCodeFixJsEnding", content);
    fourslash::unsupported("VerifyImportFixModuleSpecifiers"); // f.VerifyImportFixModuleSpecifiers(t, "", []string{"lit/decorators.js"}, &lsutil.UserPreferences{Impo
}
