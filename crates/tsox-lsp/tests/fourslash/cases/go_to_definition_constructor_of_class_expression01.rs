use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_definition_constructor_of_class_expression01() {
    let content = r#"var x = class C {
    /*definition*/constructor() {
        var other = new [|/*xusage*/C|];
    }
}

var y = class C extends x {
    constructor() {
        super();
        var other = new [|/*yusage*/C|];
    }
}
var z = class C extends x {
    m() {
        return new [|/*zusage*/C|];
    }
}

var x1 = new [|/*cref*/C|]();
var x2 = new [|/*xref*/x|]();
var y1 = new [|/*yref*/y|]();
var z1 = new [|/*zref*/z|]();"#;
    let mut s = Session::new_for_test("goToDefinitionConstructorOfClassExpression01", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "xusage", "yusage", "zusage", "cref", "xref", "yref", "zref"
}
