use tsox_lsp::fourslash::{self, Session};


#[test]
fn rename_string_literal_types5() {
    let content = r#"type T = {
    "Prop 1": string;
}

declare const fn: <K extends keyof T>(p: K) => void

fn("Prop 1"/**/)"#;
    let mut s = Session::new_for_test("renameStringLiteralTypes5", content);
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, "")
}
