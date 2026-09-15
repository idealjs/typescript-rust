use tsox_lsp::fourslash::Session;


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
    let _s = Session::new_for_test("completionEntryAfterASIExpressionInClass", content);
    // TODO: f.VerifyCompletions(t, []string{"insideid", "root"}, &fourslash.CompletionsExpectedList{
}
