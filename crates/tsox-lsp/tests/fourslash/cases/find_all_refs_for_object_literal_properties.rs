use tsox_lsp::fourslash::Session;


#[test]
fn find_all_refs_for_object_literal_properties() {
    let content = r#"var x = {
    /*1*/property: {}
};

x./*2*/property;

/*3*/let {/*4*/property: pVar} = x;"#;
    let _s = Session::new_for_test("findAllRefsForObjectLiteralProperties", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4")
}
