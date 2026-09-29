use tsox_lsp::fourslash::{self, Session};


#[test]
fn signature_help_in_function_call_on_function_declaration_in_multiple_files() {
    let content = r#"// @Filename: signatureHelpInFunctionCallOnFunctionDeclarationInMultipleFiles_file0.ts
declare function fn(x: string, y: number);
// @Filename: signatureHelpInFunctionCallOnFunctionDeclarationInMultipleFiles_file1.ts
declare function fn(x: string);
// @Filename: signatureHelpInFunctionCallOnFunctionDeclarationInMultipleFiles_file2.ts
fn(/*1*/"#;
    let mut s = Session::new_for_test("signatureHelpInFunctionCallOnFunctionDeclarationInMultipleFiles", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifySignatureHelp(t, fourslash.VerifySignatureHelpOptions{OverloadsCount: 2})
}
