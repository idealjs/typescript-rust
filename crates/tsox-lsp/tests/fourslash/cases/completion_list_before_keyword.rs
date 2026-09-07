use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_before_keyword() {
    let content = r#"// Completion after dot in named type, when the following line has a keyword
namespace TypeModule1 {
    export class C1 {}
    export class C2 {}
}
var x : TypeModule1./*TypeReference*/
namespace TypeModule2 {
    export class Test3 {}
}

// Completion after dot in named type, when the following line has a keyword
TypeModule1./*ValueReference*/
namespace TypeModule3 {
    export class Test3 {}
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, f.Markers(), &fourslash.CompletionsExpectedList{
}
