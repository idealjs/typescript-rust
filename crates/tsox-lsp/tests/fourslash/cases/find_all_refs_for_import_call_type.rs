use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_for_import_call_type() {
    let content = r#"// @Filename: /app.ts
export function he/**/llo() {};
// @Filename: /re-export.ts
export type app = typeof import("./app")
// @Filename: /indirect-use.ts
import type { app } from "./re-export";
declare const app: app
app.hello();"#;
    let mut s = Session::new_for_test("findAllRefsForImportCallType", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "")
}
