use tsox_lsp::fourslash::{self, Session};


#[test]
fn code_fix_generate_definitions() {
    let content = r#"// @Filename: /node_modules/foo/index.d.ts
module.exports = 0;
// @Filename: /a.ts
import * as foo from "foo";"#;
    let mut s = Session::new_for_test("codeFixGenerateDefinitions", content);
    // TODO: f.VerifyCodeFixNotAvailable(t)
}
