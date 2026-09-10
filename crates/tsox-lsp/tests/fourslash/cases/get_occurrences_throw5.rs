use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentHighlights"]
#[test]
fn get_occurrences_throw5() {
    let content = r#"function f(a: number) {
    try {
        throw "Hello";

        try {
            throw 10;
        }
        catch (x) {
            return 100;
        }
        finally {
            throw 10;
        }
    }
    catch (x) {
        throw "Something";
    }
    finally {
        throw "Also something";
    }
    if (a > 0) {
        return (function () {
            return;
            return;
            return;

            if (false) {
                return true;
            }
            throw "Hello!";
        })() || true;
    }

    throw 10;

    var unusued = [1, 2, 3, 4].map(x => { [|thr/**/ow|] 4 })

    return;
    return true;
    throw false;
}"#;
    let mut s = Session::new_for_test("getOccurrencesThrow5", content);
    fourslash::unsupported("VerifyBaselineDocumentHighlights"); // f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
