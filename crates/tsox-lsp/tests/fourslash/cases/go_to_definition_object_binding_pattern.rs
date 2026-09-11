use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_definition_object_binding_pattern() {
    let content = r#"
interface SomeType {
    targetProperty: number;
}

function foo(callback: (p: SomeType) => void) {}

foo(({ /*1*/targetProperty }) => {
    /*4*/targetProperty
});

let { /*2*/targetProperty }: SomeType = { /*3*/targetProperty: 42 };

let { /*5*/targetProperty: /*6*/alias_1 }: SomeType = { targetProperty: 42 };

let { x: { /*7*/targetProperty: /*8*/{} } }: { x: SomeType } = { x: { targetProperty: 42 } };"#;
    let mut s = Session::new_for_test("goToDefinitionObjectBindingPattern", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, f.MarkerNames()...)
}

#[test]
fn go_to_definition_object_binding_pattern_rest() {
    let content = r#"
interface SomeType {
    targetProperty: number;
}

let { .../*1*/rest }: SomeType = { targetProperty: 42 };"#;
    let mut s = Session::new_for_test("goToDefinitionObjectBindingPatternRest", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, f.MarkerNames()...)
}
