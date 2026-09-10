use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyImportFixModuleSpecifiers"]
#[test]
fn import_name_code_fix_infer_ending_preference_classic() {
    let content = r#"// @module: esnext
// @checkJs: true
// @allowJs: true
// @noEmit: true
// @Filename: /a.js
export const a = 0;
// @Filename: /b.js
export const b = 0;
// @Filename: /c.js
import { a } from "./a.js";

b/**/;"#;
    let mut s = Session::new_for_test("importNameCodeFixInferEndingPreference_classic", content);
    fourslash::unsupported("VerifyImportFixModuleSpecifiers"); // f.VerifyImportFixModuleSpecifiers(t, "", []string{"./b.js"}, nil /*preferences*/)
}
