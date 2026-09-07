use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineRename"]
#[test]
fn rename_string_literal_types4() {
    let content = r#"interface I {
    "Prop 1": string;
}

declare const fn: <K extends keyof I>(p: K) => void

fn("Prop 1"/**/)"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, "")
}
