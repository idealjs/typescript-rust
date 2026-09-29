use tsox_lsp::fourslash::Session;


#[test]
fn go_to_type_definition_union_type() {
    let content = r#"class /*definition0*/C {
    p;
}

interface /*definition1*/I {
    x;
}

namespace M {
    export interface /*definition2*/I {
        y;
    }
}

var x: C | I | M.I;

/*reference*/x;"#;
    let _s = Session::new_for_test("goToTypeDefinitionUnionType", content);
    // TODO: f.VerifyBaselineGoToTypeDefinition(t, "reference")
}
