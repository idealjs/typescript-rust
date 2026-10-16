use tsox_lsp::fourslash::Session;


#[test]
fn get_occurrences_abstract02() {
    let content = r#"// Not valid TS (abstract methods can only appear in abstract classes)
class Animal {
    [|abstract|] walk(): void;
    [|abstract|] makeSound(): void;
}
// abstract cannot appear here, won't get highlighted
let c = /*1*/abstract class Foo {
    /*2*/abstract foo(): void;
    abstract bar(): void;
}"#;
    let _s = Session::new_for_test("getOccurrencesAbstract02", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, "1", "2")
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
