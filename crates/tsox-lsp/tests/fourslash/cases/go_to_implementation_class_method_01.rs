use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_implementation_class_method_01() {
    let content = r#"abstract class AbstractBar {
    abstract he/*declaration*/llo(): void;
}

class Bar extends AbstractBar{
    [|hello|]() {}
}

function whatever(x: AbstractBar) {
    x.he/*reference*/llo();
}"#;
    let mut s = Session::new_for_test("goToImplementationClassMethod_01", content);
    // TODO: f.VerifyBaselineGoToImplementation(t, "reference", "declaration")
}
