use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn code_fix_spelling5() {
    let content = r#"// @Filename: f1.ts
export const fooooooooo = 1;
// @Filename: f2.ts
import {[|fooooooooa|]} from "./f1"; fooooooooa;"#;
    let mut s = Session::new_for_test("codeFixSpelling5", content);
    fourslash::go_to_file(&mut s, "f2.ts");
    // TODO: f.VerifyRangeAfterCodeFix(t, `fooooooooo`, false, 0, 0)
}
