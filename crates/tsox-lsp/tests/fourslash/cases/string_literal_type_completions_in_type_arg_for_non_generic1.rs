use tsox_lsp::fourslash::Session;


#[test]
fn string_literal_type_completions_in_type_arg_for_non_generic1() {
    let content = r#"interface Foo {}
type Bar = {};

let x: Foo<"/*1*/">;
let y: Bar<"/*2*/">;"#;
    let _s = Session::new_for_test("stringLiteralTypeCompletionsInTypeArgForNonGeneric1", content);
    // TODO: f.VerifyCompletions(t, f.Markers(), &fourslash.CompletionsExpectedList{
}
