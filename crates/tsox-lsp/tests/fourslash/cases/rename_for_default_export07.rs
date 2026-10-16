use tsox_lsp::fourslash::{self, Session};


#[test]
fn rename_for_default_export07() {
    let content = r#"// @Filename: foo.ts
export default function /**/[|DefaultExportedFunction|]() {
    return DefaultExportedFunction
}
/**
 *  Commenting DefaultExportedFunction
 */

var x: typeof DefaultExportedFunction;

var y = DefaultExportedFunction();"#;
    let mut s = Session::new_for_test("renameForDefaultExport07", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyRenameSucceeded(t, nil /*preferences*/)
}
