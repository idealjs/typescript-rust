use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_inside_templates2() {
    let content = r#"/*1*/function /*2*/f(...rest: any[]) { }
/*3*/f ` + "`" + `${ /*4*/f } ${ /*5*/f }` + "`" + `"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4", "5")
}
