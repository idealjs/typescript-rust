use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_modifiers() {
    let content = r#"// @Filename: /a.ts
/*export*/export class A/*A*/ {

    /*private*/private z/*z*/: string;

    /*readonly*/readonly x/*x*/: string;

    /*async*/async a/*a*/() {  }

    /*override*/override b/*b*/() {}

    /*public1*/public/*public2*/ as/*multipleModifiers*/ync c/*c*/() { }
}

exp/*exportFunction*/ort function foo/*foo*/() { }"#;
    let _s = Session::new_for_test("goToDefinitionModifiers", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "export", "A", "private", "z", "readonly", "x", "async", "a"
}
