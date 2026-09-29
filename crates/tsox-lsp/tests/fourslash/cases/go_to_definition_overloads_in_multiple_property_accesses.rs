use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_overloads_in_multiple_property_accesses() {
    let content = r#"namespace A {
    export namespace B {
        export function f(value: number): void;
        export function /*1*/f(value: string): void;
        export function f(value: number | string) {}
    }
}
A.B.[|/*2*/f|]("");"#;
    let _s = Session::new_for_test("goToDefinitionOverloadsInMultiplePropertyAccesses", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "2")
}
