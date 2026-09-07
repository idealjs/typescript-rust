use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn quick_info_for_shorthand_property() {
    let content = r#"// @strict: false
var name1 = undefined, id1 = undefined;
var /*obj1*/obj1 = {/*name1*/name1, /*id1*/id1};
var name2 = "Hello";
var id2 = 10000;
var /*obj2*/obj2 = {/*name2*/name2, /*id2*/id2};"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "obj1", "var obj1: {\n    name1: any;\n    id1: any;\n}", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "name1", "(property) name1: any", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "id1", "(property) id1: any", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "obj2", "var obj2: {\n    name2: string;\n    id2: number;\n}", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "name2", "(property) name2: string", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "id2", "(property) id2: number", "")
}
