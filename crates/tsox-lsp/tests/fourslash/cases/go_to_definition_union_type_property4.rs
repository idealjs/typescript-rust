use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_union_type_property4() {
    let content = r#"interface SnapCrackle {
    /*def1*/pop(): string;
}

interface Magnitude {
    /*def2*/pop(): number;
}

interface Art {
    /*def3*/pop(): boolean;
}

var art: Art;
var magnitude: Magnitude;
var snapcrackle: SnapCrackle;

var x = (snapcrackle || magnitude || art).[|/*usage*/pop|];"#;
    let _s = Session::new_for_test("goToDefinitionUnionTypeProperty4", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "usage")
}
