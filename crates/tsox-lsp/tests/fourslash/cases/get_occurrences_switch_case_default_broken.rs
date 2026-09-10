use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentHighlights"]
#[test]
fn get_occurrences_switch_case_default_broken() {
    let content = r#"swi/*1*/tch(10) {
    case 1:
    case 2:
    c/*2*/ase 4:
    case 8:
    case 0xBEEF:
    de/*4*/fult:
        break;
    /*5*/cas 16:
    c/*3*/ase 12:
        function f() {
            br/*11*/eak;
            /*12*/break;
        }
}

sw/*6*/itch (10) {
    de/*7*/fault
    case 1:
    case 2

    c/*8*/ose 4:
    case 8:
    case 0xBEEF:
        bre/*9*/ak;
    case 16:
        () => bre/*10*/ak;
}"#;
    let mut s = Session::new_for_test("getOccurrencesSwitchCaseDefaultBroken", content);
    fourslash::unsupported("VerifyBaselineDocumentHighlights"); // f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Markers())...)
}
