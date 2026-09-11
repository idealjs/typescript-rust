use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_implementation_local_05() {
    let content = r#"class Bar {
    public hello() {}
}

var [|someVar|] = new Bar();
someVa/*reference*/r.hello();"#;
    let mut s = Session::new_for_test("goToImplementationLocal_05", content);
    // TODO: f.VerifyBaselineGoToImplementation(t, "reference")
}
