use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_for_shorthand_property() {
    let content = r#"// @strict: false
var name1 = undefined, id1 = undefined;
var /*obj1*/obj1 = {/*name1*/name1, /*id1*/id1};
var name2 = "Hello";
var id2 = 10000;
var /*obj2*/obj2 = {/*name2*/name2, /*id2*/id2};"#;
    let mut s = Session::new_for_test("quickInfoForShorthandProperty", content);
    fourslash::verify_quick_info_at(&mut s, "obj1", "var obj1: {\n    name1: any;\n    id1: any;\n}", "");
    fourslash::verify_quick_info_at(&mut s, "name1", "(property) name1: any", "");
    fourslash::verify_quick_info_at(&mut s, "id1", "(property) id1: any", "");
    fourslash::verify_quick_info_at(&mut s, "obj2", "var obj2: {\n    name2: string;\n    id2: number;\n}", "");
    fourslash::verify_quick_info_at(&mut s, "name2", "(property) name2: string", "");
    fourslash::verify_quick_info_at(&mut s, "id2", "(property) id2: number", "");
}
