use tsox_lsp::fourslash::Session;


#[test]
fn get_occurrences_abstract03() {
    let content = r#"function f() {
    [|abstract|] class A {
        [|abstract|] m(): void;
    }
    abstract class B {}
}
switch (0) {
    case 0:
        [|abstract|] class A { [|abstract|] m(): void; }
    default:
        [|abstract|] class B { [|abstract|] m(): void; }
}"#;
    let _s = Session::new_for_test("getOccurrencesAbstract03", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
