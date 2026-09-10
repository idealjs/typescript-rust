use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_in_function_declaration() {
    let content = r#"// @lib: es5
var a = 0;
function foo(/**/"#;
    let mut s = Session::new_for_test("completionListInFunctionDeclaration", content);
    fourslash::verify_completions_empty_at(&mut s, Some(""));
    fourslash::insert(&mut s, "a");
    fourslash::verify_completions_empty_at(&mut s, None);
    fourslash::insert(&mut s, " , ");
    fourslash::verify_completions_empty_at(&mut s, None);
    fourslash::insert(&mut s, "b");
    fourslash::verify_completions_empty_at(&mut s, None);
    fourslash::insert(&mut s, ":");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "number, ");
    fourslash::verify_completions_empty_at(&mut s, None);
}
