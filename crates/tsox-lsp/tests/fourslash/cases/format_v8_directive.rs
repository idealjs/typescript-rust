use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn format_v8_directive() {
    let content = r#"// @Filename: foo.js
function foo() {}
/*1*/%PrepareFunctionForOptimization(foo)/*2*/;"#;
    let mut s = Session::new_for_test("formatV8Directive", content);
    fourslash::format_selection(&mut s, "1", "2");
}
