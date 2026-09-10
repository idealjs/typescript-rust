use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn js_doc_property_description9() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"class LiteralClass {
    /** Something generic */
    static [key: ` + "`" + `prefix${string}` + "`" + `]: any;
    /** Something else */
    static [key: ` + "`" + `prefix${number}` + "`" + `]: number;
}
function literalClass(e: typeof LiteralClass) {
    console.log(e./*literal1Class*/prefixMember); 
    console.log(e./*literal2Class*/anything);
    console.log(e./*literal3Class*/prefix0);
}"#;
    let mut s = Session::new_for_test("jsDocPropertyDescription9", content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "literal1Class", "(index) LiteralClass[`prefix${string}`]: any", "Something g
    fourslash::verify_quick_info_at(&mut s, "literal2Class", "any", "");
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "literal3Class", "(index) LiteralClass[`prefix${string}` | `prefix${number}`]
}
