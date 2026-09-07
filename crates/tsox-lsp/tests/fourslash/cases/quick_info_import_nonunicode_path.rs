use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn quick_info_import_nonunicode_path() {
    let content = r#"// @Filename: /江南今何在/tmp.ts
export const foo = 1;
// @Filename: /test.ts
import { foo } from "./江南/*1*/今何在/tmp";"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "module \"./江南今何在/tmp\"", "")
}
