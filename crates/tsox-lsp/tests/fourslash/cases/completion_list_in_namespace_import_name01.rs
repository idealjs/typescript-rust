use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_in_namespace_import_name01() {
    let content = r#"// @Filename: m1.ts
export var foo: number = 1;
// @Filename: m2.ts
import * as /**/ from "m1""#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", nil)
}
