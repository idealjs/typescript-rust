use tsox_lsp::fourslash::{self, Session};


#[test]
fn member_list_of_module_before_keyword() {
    let content = r#"namespace TypeModule1 {
    export class C1 { }
    export class C2 { }
}
var x: TypeModule1./*namedType*/
namespace TypeModule2 {
    export class Test3 {}
}

TypeModule1./*dottedExpression*/
namespace TypeModule3 {
    export class Test3 {}
}"#;
    let mut s = Session::new_for_test("memberListOfModuleBeforeKeyword", content);
    // TODO: f.VerifyCompletions(t, f.Markers(), &fourslash.CompletionsExpectedList{
}
