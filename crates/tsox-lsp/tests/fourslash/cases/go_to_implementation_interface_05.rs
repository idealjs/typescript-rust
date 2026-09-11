use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_implementation_interface_05() {
    let content = r#"interface Fo/*interface_definition*/o {
    (a: number): void
}

let bar2 = <Foo> [|function(a) {}|];
"#;
    let mut s = Session::new_for_test("goToImplementationInterface_05", content);
    // TODO: f.VerifyBaselineGoToImplementation(t, "interface_definition")
}
