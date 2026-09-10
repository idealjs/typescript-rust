use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_class_with_static_this_access() {
    let content = r#"[|class /*0*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 0 |}C|] {
    static s() {
        /*1*/[|this|];
    }
    static get f() {
        return /*2*/[|this|];

        function inner() { this; }
        class Inner { x = this; }
    }
}|]"#;
    let mut s = Session::new_for_test("findAllRefsClassWithStaticThisAccess", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "0", "1", "2")
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[1])
}
