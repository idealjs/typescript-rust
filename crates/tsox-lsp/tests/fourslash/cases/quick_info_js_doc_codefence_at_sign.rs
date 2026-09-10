use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: * <div></div>"]
#[test]
fn quick_info_js_doc_codefence_at_sign() {
    let content = r#"/**
 * text
 * @example Foo
 * ` + "```" + "#;
    // TODO: * @Embed[asfasdfasf]
    // TODO: * ` + "```" + `
    // TODO: * <div></div>
    // TODO: * ` + "```" + `
    // TODO: * @tag inside code
    // TODO: * ` + "```" + `
    let mut s = Session::new_for_test("quickInfoJSDocCodefenceAtSign", content);
    fourslash::unsupported("VerifyBaselineHover"); // f.VerifyBaselineHover(t)
}
