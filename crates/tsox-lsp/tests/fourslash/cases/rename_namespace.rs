use tsox_lsp::fourslash::Session;


#[test]
fn rename_namespace() {
    let content = r#"namespace /**/NS {
    export const enum E {
        A = 'a'
    }
}

const a: NS.E = NS.E.A;"#;
    let _s = Session::new_for_test("renameNamespace", content);
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, "")
}
