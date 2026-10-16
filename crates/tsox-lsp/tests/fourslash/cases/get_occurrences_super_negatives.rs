use tsox_lsp::fourslash::Session;


#[test]
fn get_occurrences_super_negatives() {
    let content = r#"function f(x = [|super|]) {
    [|super|];
}

namespace M {
    [|super|];
    function f(x = [|super|]) {
    [|super|];
    }

    class A {
    }

    class B extends A {
        constructor() {
            super();
        }
    }
}"#;
    let _s = Session::new_for_test("getOccurrencesSuperNegatives", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
