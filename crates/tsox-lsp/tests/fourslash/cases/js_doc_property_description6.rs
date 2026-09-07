use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn js_doc_property_description6() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"interface Literal1Example {
    [key: ` + "`" + `prefix${string}` + "`" + `]: number | string;
    /** Something else */
    [key: ` + "`" + `prefix${number}` + "`" + `]: number;
}
function literal1Example(e: Literal1Example) {
    console.log(e./*literal1*/prefixMember);
    console.log(e./*literal2*/anything);
    console.log(e./*literal3*/prefix0);
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "literal1", "(index) Literal1Example[`prefix${string}`]: string | number", ""
    fourslash::verify_quick_info_at(&mut s, "literal2", "any", "");
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "literal3", "(index) Literal1Example[`prefix${string}` | `prefix${number}`]: 
}
