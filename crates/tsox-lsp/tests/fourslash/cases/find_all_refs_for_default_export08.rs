use tsox_lsp::fourslash::{self, Session};


#[test]
fn find_all_refs_for_default_export08() {
    let content = r#"export default class DefaultExportedClass {
}

var x: DefaultExportedClass;

var y = new DefaultExportedClass;

namespace /*1*/DefaultExportedClass {
}"#;
    let mut s = Session::new_for_test("findAllRefsForDefaultExport08", content);
    fourslash::verify_no_errors(&mut s, );
    // TODO: f.VerifyBaselineFindAllReferences(t, "1")
}
