use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_implementation_interface_01() {
    let content = r#"interface Fo/*interface_definition*/o { hello(): void }

class [|SuperBar|] implements Foo {
    hello () {}
}

abstract class [|AbstractBar|] implements Foo {
    abstract hello (): void;
}

class [|Bar|] extends SuperBar {
}

class [|NotAbstractBar|] extends AbstractBar {
    hello () {}
}

var x = new SuperBar();
var y: SuperBar = new SuperBar();
var z: AbstractBar = new NotAbstractBar();"#;
    let mut s = Session::new_for_test("goToImplementationInterface_01", content);
    // TODO: f.VerifyBaselineGoToImplementation(t, "interface_definition")
}
