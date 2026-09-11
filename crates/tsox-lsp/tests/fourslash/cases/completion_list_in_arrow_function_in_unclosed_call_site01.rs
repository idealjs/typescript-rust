use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_arrow_function_in_unclosed_call_site01() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"declare function foo(...params: any[]): any;
function getAllFiles(rootFileNames: string[]) {
    var processedFiles = rootFileNames.map(fileName => foo(/*1*/"#;
    let mut s = Session::new_for_test("completionListInArrowFunctionInUnclosedCallSite01", content);
    fourslash::verify_completions_include_exclude_at(&mut s, Some("1"), &["fileName", "rootFileNames", "getAllFiles", "foo"], &[]);
    // TODO: }
}
