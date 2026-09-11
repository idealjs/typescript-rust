use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_for_string_literal_nonrelative_import18() {
    let content = r#"// @Filename: tsconfig.json
{
    "compilerOptions": {
       "paths": {
           "/*": ["./*"]
       },
    }
}
// @Filename: test0.ts
import * as foo1 from "/path/w/*first*/
// @Filename: path/whatever.ts
export {}"#;
    let mut s = Session::new_for_test("completionForStringLiteralNonrelativeImport18", content);
    // TODO: f.VerifyCompletions(t, []string{"first"}, &fourslash.CompletionsExpectedList{
}
