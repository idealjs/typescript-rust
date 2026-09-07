use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
