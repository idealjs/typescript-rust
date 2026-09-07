use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentHighlights"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineDocumentHighlights"); // f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
