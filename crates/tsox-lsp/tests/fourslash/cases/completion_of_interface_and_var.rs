use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_of_interface_and_var() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @lib: es5
interface AnalyserNode {
}
declare var AnalyserNode: {
    prototype: AnalyserNode;
    new(): AnalyserNode;
};
/**/"#;
    let mut s = Session::new_for_test("completionOfInterfaceAndVar", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
