use tsox_lsp::fourslash::Session;


#[test]
fn get_occurrences_abstract01() {
    let content = r#"[|abstract|] class Animal {
    [|abstract|] prop1; // Does not compile
    [|abstract|] abstract();
    [|abstract|] walk(): void;
    [|abstract|] makeSound(): void;
}
// Abstract class below should not get highlighted
abstract class Foo {
    abstract foo(): void;
    abstract bar(): void;
}"#;
    let _s = Session::new_for_test("getOccurrencesAbstract01", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
