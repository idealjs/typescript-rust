use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentHighlights"]
#[test]
fn get_occurrences_constructor2() {
    let content = r#"class C {
    constructor();
    constructor(x: number);
    constructor(y: string, x: number);
    constructor(a?: any, ...r: any[]) {
        if (a === undefined && r.length === 0) {
            return;
        }

        return;
    }
}

class D {
    [|con/**/structor|](public x: number, public y: number) {
    }
}"#;
    let mut s = Session::new_for_test("getOccurrencesConstructor2", content);
    fourslash::unsupported("VerifyBaselineDocumentHighlights"); // f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
