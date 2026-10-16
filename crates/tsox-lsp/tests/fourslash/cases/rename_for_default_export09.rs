use tsox_lsp::fourslash::{self, Session};


#[test]
fn rename_for_default_export09() {
    let content = r#"// @Filename: foo.ts
function /**/[|f|]() {
    return 100;
}

export default f;

var x: typeof f;

var y = f();

/**
 *  Commenting f
 */
namespace f {
    var local = 100;
}"#;
    let mut s = Session::new_for_test("renameForDefaultExport09", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyRenameSucceeded(t, nil /*preferences*/)
}
