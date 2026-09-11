use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn format_v8_directive() {
    let content = r#"// @Filename: foo.js
function foo() {}
/*1*/%PrepareFunctionForOptimization(foo)/*2*/;"#;
    let mut s = Session::new_for_test("formatV8Directive", content);
    // TODO: f.FormatSelection(t, "1", "2")
}
