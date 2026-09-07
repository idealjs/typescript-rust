use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyRenameSucceeded"]
#[test]
fn rename_for_default_export08() {
    let content = r#"// @Filename: foo.ts
export default function DefaultExportedFunction() {
    return /**/[|DefaultExportedFunction|]
}
/**
 *  Commenting DefaultExportedFunction
 */

var x: typeof DefaultExportedFunction;

var y = DefaultExportedFunction();"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyRenameSucceeded"); // f.VerifyRenameSucceeded(t, nil /*preferences*/)
}
