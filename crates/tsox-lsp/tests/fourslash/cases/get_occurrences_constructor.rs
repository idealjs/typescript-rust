use tsox_lsp::fourslash::Session;


#[test]
fn get_occurrences_constructor() {
    let content = r#"class C {
    [|const/**/ructor|]();
    [|constructor|](x: number);
    [|constructor|](y: string, x: number);
    [|constructor|](a?: any, ...r: any[]) {
        if (a === undefined && r.length === 0) {
            return;
        }

        return;
    }
}

class D {
    constructor(public x: number, public y: number) {
    }
}"#;
    let _s = Session::new_for_test("getOccurrencesConstructor", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
