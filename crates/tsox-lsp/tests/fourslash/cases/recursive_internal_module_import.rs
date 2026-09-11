use tsox_lsp::fourslash::{self, Session};


#[test]
fn recursive_internal_module_import() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"namespace M {
    import A = B;
    import /**/B = A;
}
"#;
    let mut s = Session::new_for_test("recursiveInternalModuleImport", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyQuickInfoExists(t)
}
