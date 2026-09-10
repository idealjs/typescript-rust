use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_in_type_parameter_of_class_expression1() {
    let content = r#"// @lib: es5
var C0 = class D</*0*/
var C1 = class D</*1*/T> {}
var C2 = class D<T, /*2*/
var C3 = class D<T, /*3*/U>{}
var C4 = class D<T extends /*4*/>{}"#;
    let mut s = Session::new_for_test("completionListInTypeParameterOfClassExpression1", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"0", "1", "2", "3"}, nil)
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "4", &fourslash.CompletionsExpectedList{
}
