use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_in_comments3() {
    let content = r#"// @lib: es5
 /*{| "name": "1" |}
 /*  {| "name": "2" |}
 /*  *{| "name": "3" |}
 /*  */{| "name": "4" |}
 {| "name": "5" |}/*  */
/* {| "name": "6" |}"#;
    let mut s = Session::new_for_test("completionListInComments3", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"1", "2", "3", "6"}, nil)
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"4", "5"}, &fourslash.CompletionsExpectedList{
}
