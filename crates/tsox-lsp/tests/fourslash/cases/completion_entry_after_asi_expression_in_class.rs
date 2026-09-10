use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_entry_after_asi_expression_in_class() {
    let content = r#"class Parent {
  protected shouldWork() {
      console.log();
  }
}

class Child extends Parent {
            // this assumes ASI, but on next line wants to  
  x = () => 1
  shoul/*insideid*/ 
}

class ChildTwo extends Parent {
            // this assumes ASI, but on next line wants to  
  x = () => 1
  /*root*/ //nothing
}"#;
    let mut s = Session::new_for_test("completionEntryAfterASIExpressionInClass", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"insideid", "root"}, &fourslash.CompletionsExpectedList{
}
