use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineRename"]
#[test]
fn rename_namespace() {
    let content = r#"namespace /**/NS {
    export const enum E {
        A = 'a'
    }
}

const a: NS.E = NS.E.A;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, "")
}
