use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn alias_merging_with_namespace() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"namespace bar { }
import bar = bar/**/;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "", "namespace bar\nimport bar = bar", "")
}
