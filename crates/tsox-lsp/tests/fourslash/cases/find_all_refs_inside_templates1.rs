use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_inside_templates1() {
    let content = r#"/*1*/var /*2*/x = 10;
var y = ` + "`" + `${ /*3*/x } ${ /*4*/x }` + "`" + `"#;
    let mut s = Session::new_for_test("findAllRefsInsideTemplates1", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4")
}
