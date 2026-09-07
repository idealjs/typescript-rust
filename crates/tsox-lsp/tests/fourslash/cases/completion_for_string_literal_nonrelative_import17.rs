use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_for_string_literal_nonrelative_import17() {
    let content = r#"// @Filename: tsconfig.json
{
    "compilerOptions": {
        "paths": {
            "module1/*": ["some/path/*"],
        }
    }
}
// @Filename: test0.ts
import * as foo1 from "module1/w/*first*/
// @Filename: some/path/whatever.ts
export {}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"first"}, &fourslash.CompletionsExpectedList{
}
