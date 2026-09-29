use tsox_lsp::fourslash::Session;


#[test]
fn go_to_definition_object_literal_properties1() {
    let content = r#"interface PropsBag {
   /*first*/propx: number
}
function foo(arg: PropsBag) {}
foo({
   [|pr/*p1*/opx|]: 10
})
function bar(firstarg: boolean, secondarg: PropsBag) {}
bar(true, {
   [|pr/*p2*/opx|]: 10
})"#;
    let _s = Session::new_for_test("goToDefinitionObjectLiteralProperties1", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "p1", "p2")
}
