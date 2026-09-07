use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn js_doc_property_description5() {
    let content = r#"interface Multiple1Example {
    /** Something generic */
    [key: number | symbol | ` + "`" + `data-${string}` + "`" + ` | ` + "`" + `data-${number}` + "`" + `]: string;
}
function multiple1Example(e: Multiple1Example) {
    console.log(e./*multiple1*/anything);
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "multiple1", "any", "")
}
