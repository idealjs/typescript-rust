use tsox_lsp::fourslash::Session;


#[test]
fn find_all_refs_for_computed_properties() {
    let content = r#"interface I {
    ["/*0*/prop1"]: () => void;
}

class C implements I {
    ["/*1*/prop1"]: any;
}

var x: I = {
    ["/*2*/prop1"]: function () { },
}"#;
    let _s = Session::new_for_test("findAllRefsForComputedProperties", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "0", "1", "2")
}
