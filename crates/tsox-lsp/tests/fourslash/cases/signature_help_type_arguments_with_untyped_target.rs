use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.GoToEachMarker"]
#[test]
fn signature_help_on_type_arguments_with_unresolved_target() {
    let content = r#"
/*1*/un/*2*/resolvedVal/*3*/</*4*/Un/*5*/resolvedType/*6*/>/*7*/(/*8*/un/*9*/resolvedVal/*10*/);
"#;
    let mut s = Session::new_for_test("signatureHelpOnTypeArgumentsWithUnresolvedTarget", content);
    fourslash::unsupported("GoToEachMarker"); // f.GoToEachMarker(t, nil, func(marker *fourslash.Marker, index int) {
}
