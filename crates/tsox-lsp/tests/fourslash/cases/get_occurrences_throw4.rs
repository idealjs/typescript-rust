use tsox_lsp::fourslash::Session;


#[test]
fn get_occurrences_throw4() {
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
            [|return|];
            [|return|];
            [|return|];

            if (false) {
                [|return|] true;
            }
            [|th/**/row|] "Hello!";
        })() || true;
    }

    throw 10;

    var unusued = [1, 2, 3, 4].map(x => { throw 4 })

    return;
    return true;
    throw false;
}"#;
    let _s = Session::new_for_test("getOccurrencesThrow4", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
