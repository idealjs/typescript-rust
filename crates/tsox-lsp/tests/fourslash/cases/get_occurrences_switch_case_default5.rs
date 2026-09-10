use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentHighlights"]
#[test]
fn get_occurrences_switch_case_default5() {
    let content = r#"switch/*1*/ (10) {
    case/*2*/ 1:
    case/*3*/ 2:
    case/*4*/ 4:
    case/*5*/ 8:
        foo: switch/*6*/ (20) {
            case/*7*/ 1:
            case/*8*/ 2:
                break/*9*/;
            default/*10*/:
                break foo;
        }
    case/*11*/ 0xBEEF:
    default/*12*/:
        break/*13*/;
    case 16/*14*/:
}"#;
    let mut s = Session::new_for_test("getOccurrencesSwitchCaseDefault5", content);
    fourslash::unsupported("VerifyBaselineDocumentHighlights"); // f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Markers())...)
}
