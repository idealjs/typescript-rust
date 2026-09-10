use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_for_computed_properties2() {
    let content = r#"interface I {
    [/*1*/42](): void;
}

class C implements I {
    [/*2*/42]: any;
}

var x: I = {
    ["/*3*/42"]: function () { }
}"#;
    let mut s = Session::new_for_test("findAllRefsForComputedProperties2", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
