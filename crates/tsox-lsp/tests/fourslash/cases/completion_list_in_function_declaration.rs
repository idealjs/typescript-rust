use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_in_function_declaration() {
    let content = r#"// @lib: es5
var a = 0;
function foo(/**/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", nil)
    fourslash::insert(&mut s, "a");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, nil, nil)
    fourslash::insert(&mut s, " , ");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, nil, nil)
    fourslash::insert(&mut s, "b");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, nil, nil)
    fourslash::insert(&mut s, ":");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "number, ");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, nil, nil)
}
