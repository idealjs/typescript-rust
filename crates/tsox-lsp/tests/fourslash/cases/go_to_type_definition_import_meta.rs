use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_type_definition_import_meta() {
    let content = r#"// @lib: es5
// @module: esnext
// @Filename: foo.ts
/// <reference path='./bar.d.ts' />
import.me/*reference*/ta;
//@Filename: bar.d.ts
interface /*definition*/ImportMeta {
}"#;
    let mut s = Session::new_for_test("goToTypeDefinitionImportMeta", content);
    // TODO: f.VerifyBaselineGoToTypeDefinition(t, "reference")
}
