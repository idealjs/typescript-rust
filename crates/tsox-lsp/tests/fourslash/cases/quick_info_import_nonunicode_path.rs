use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_import_nonunicode_path() {
    let content = r#"// @Filename: /江南今何在/tmp.ts
export const foo = 1;
// @Filename: /test.ts
import { foo } from "./江南/*1*/今何在/tmp";"#;
    let mut s = Session::new_for_test("quickInfoImportNonunicodePath", content);
    fourslash::verify_quick_info_at(&mut s, "1", "module \"./江南今何在/tmp\"", "");
}
