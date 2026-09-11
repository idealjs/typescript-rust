use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_definition_import_meta() {
    let content = r#"// @module: esnext
// @Filename: foo.ts
/// <reference path='./bar.d.ts' />
import.me/*reference*/ta;
//@Filename: bar.d.ts
interface ImportMeta {
}"#;
    let mut s = Session::new_for_test("goToDefinitionImportMeta", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "reference")
    fourslash::verify_no_errors(&mut s, );
}
