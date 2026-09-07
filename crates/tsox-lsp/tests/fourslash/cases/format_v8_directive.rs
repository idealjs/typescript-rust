use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn format_v8_directive() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @Filename: foo.js
function foo() {}
/*1*/%PrepareFunctionForOptimization(foo)/*2*/;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("FormatSelection"); // f.FormatSelection(t, "1", "2")
}
