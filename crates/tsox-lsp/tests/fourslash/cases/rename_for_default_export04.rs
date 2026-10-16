use tsox_lsp::fourslash::{self, Session};


#[test]
fn rename_for_default_export04() {
    let content = r#"// @Filename: foo.ts
export default class /**/[|DefaultExportedClass|] {
}
/*
 *  Commenting DefaultExportedClass
 */

var x: DefaultExportedClass;

var y = new DefaultExportedClass;"#;
    let mut s = Session::new_for_test("renameForDefaultExport04", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyRenameSucceeded(t, nil /*preferences*/)
}
