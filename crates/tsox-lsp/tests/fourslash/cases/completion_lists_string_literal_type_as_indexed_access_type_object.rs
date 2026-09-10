use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn completion_lists_string_literal_type_as_indexed_access_type_object() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"let firstCase: "a/*case_1*/"["foo"]
let secondCase: "b/*case_2*/"["bar"]
let thirdCase: "c/*case_3*/"["baz"]
let fourthCase: "en/*case_4*/"["qux"]
interface Foo {
 bar: string;
 qux: string;
}
let fifthCase: Foo["b/*case_5*/"]
let sixthCase: Foo["qu/*case_6*/"]"#;
    let mut s = Session::new_for_test("completionListsStringLiteralTypeAsIndexedAccessTypeObject", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"case_1", "case_2", "case_3", "case_4"}, nil)
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "case_5", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "case_6", &fourslash.CompletionsExpectedList{
}
