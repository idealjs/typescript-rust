use tsox_lsp::fourslash::{self, Session};

#[test]
fn quick_info_for_object_binding_element_property_name04() {
    let content = r#"interface Recursive {
    next?: Recursive;
    value: any;
}

function f ({ /*1*/next: { /*2*/next: x} }) {
}"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(&mut s, "1", "(property) next: {\n    next: any;\n}", "");
    fourslash::verify_quick_info_at(&mut s, "2", "(property) next: any", "");
}
