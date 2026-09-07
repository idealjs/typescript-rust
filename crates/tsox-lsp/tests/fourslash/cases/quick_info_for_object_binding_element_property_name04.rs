use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn quick_info_for_object_binding_element_property_name04() {
    let content = r#"interface Recursive {
    next?: Recursive;
    value: any;
}

function f ({ /*1*/next: { /*2*/next: x} }) {
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "(property) next: {\n    next: any;\n}", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "(property) next: any", "")
}
