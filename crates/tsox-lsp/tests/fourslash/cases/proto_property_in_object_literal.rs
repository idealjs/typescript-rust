use tsox_lsp::fourslash::{self, Session};


#[test]
fn proto_property_in_object_literal() {
    let content = r#"var o1 = {
    "__proto__": 10
};
var o2 = {
    __proto__: 10
};
o1./*1*/
o2./*2*/"#;
    let mut s = Session::new_for_test("protoPropertyInObjectLiteral", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "__proto__ = 10;");
    fourslash::verify_quick_info_at(&mut s, "1", "(property) \"__proto__\": number", "");
    fourslash::go_to_marker(&mut s, "2");
    // TODO: f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "__proto__ = 10;");
    fourslash::verify_quick_info_at(&mut s, "2", "(property) __proto__: number", "");
}
