use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_for_default_export03() {
    let content = r#"/*1*/function /*2*/f() {
    return 100;
}

/*3*/export default /*4*/f;

var x: typeof /*5*/f;

var y = /*6*/f();

/*7*/namespace /*8*/f {
    var local = 100;
}"#;
    let mut s = Session::new_for_test("findAllRefsForDefaultExport03", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4", "5", "6", "7", "8")
}
