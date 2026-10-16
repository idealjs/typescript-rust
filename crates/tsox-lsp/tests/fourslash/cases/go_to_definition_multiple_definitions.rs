use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_multiple_definitions() {
    let content = r#"// @Filename: a.ts
interface /*interfaceDefinition1*/IFoo {
    instance1: number;
}
// @Filename: b.ts
interface /*interfaceDefinition2*/IFoo {
    instance2: number;
}

interface /*interfaceDefinition3*/IFoo {
    instance3: number;
}

var ifoo: [|IFo/*interfaceReference*/o|];
// @Filename: c.ts
module /*moduleDefinition1*/Module {
    export class c1 { }
}
// @Filename: d.ts
module /*moduleDefinition2*/Module {
    export class c2 { }
}
// @Filename: e.ts
[|Modul/*moduleReference*/e|];"#;
    let _s = Session::new_for_test("goToDefinitionMultipleDefinitions", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "interfaceReference", "moduleReference")
}
