use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyRenameSucceeded"]
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
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyRenameSucceeded"); // f.VerifyRenameSucceeded(t, nil /*preferences*/)
}
