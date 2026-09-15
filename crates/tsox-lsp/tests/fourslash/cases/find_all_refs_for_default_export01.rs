use tsox_lsp::fourslash::Session;


#[test]
fn find_all_refs_for_default_export01() {
    let content = r#"/*1*/export default class /*2*/DefaultExportedClass {
}

var x: /*3*/DefaultExportedClass;

var y = new /*4*/DefaultExportedClass;"#;
    let _s = Session::new_for_test("findAllRefsForDefaultExport01", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4")
}
