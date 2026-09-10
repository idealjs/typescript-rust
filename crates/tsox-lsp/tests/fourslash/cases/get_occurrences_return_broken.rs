use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentHighlights"]
#[test]
fn get_occurrences_return_broken() {
    let content = r#"ret/*1*/urn;
retu/*2*/rn;
function f(a: number) {
    if (a > 0) {
        return (function () {
            () => [|return|];
            [|return|];
            [|return|];

            if (false) {
                [|return|] true;
            }
        })() || true;
    }

    var unusued = [1, 2, 3, 4].map(x => { return 4 })

    return;
    return true;
}

class A {
    ret/*3*/urn;
    r/*4*/eturn 8675309;
}"#;
    let mut s = Session::new_for_test("getOccurrencesReturnBroken", content);
    fourslash::unsupported("VerifyBaselineDocumentHighlights"); // f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
    fourslash::unsupported("VerifyBaselineDocumentHighlights"); // f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Markers())...)
}
