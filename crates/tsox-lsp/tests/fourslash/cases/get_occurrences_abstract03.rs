use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentHighlights"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineDocumentHighlights"); // f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
