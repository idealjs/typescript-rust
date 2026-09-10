use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentHighlights"]
#[test]
fn get_occurrences_loop_break_continue() {
    let content = r#"var arr = [1, 2, 3, 4];
label1: [|for|] (var n in arr) {
    [|break|];
    [|continue|];
    [|br/**/eak|] label1;
    [|continue|] label1;

    label2: for (var i = 0; i < arr[n]; i++) {
        [|break|] label1;
        [|continue|] label1;

        break;
        continue;
        break label2;
        continue label2;

        function foo() {
            label3: while (true) {
                break;
                continue;
                break label3;
                continue label3;

                // these cross function boundaries
                break label1;
                continue label1;
                break label2;
                continue label2;

                label4: do {
                    break;
                    continue;
                    break label4;
                    continue label4;

                    break label3;
                    continue label3;

                    switch (10) {
                        case 1:
                        case 2:
                            break;
                            break label4;
                        default:
                            continue;
                    }

                    // these cross function boundaries
                    break label1;
                    continue label1;
                    break label2;
                    continue label2;
                    () => { break; }
                } while (true)
            }
        }
    }
}

label5: while (true) break label5;

label7: while (true) continue label5;"#;
    let mut s = Session::new_for_test("getOccurrencesLoopBreakContinue", content);
    fourslash::unsupported("VerifyBaselineDocumentHighlights"); // f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
