use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_for_module_global() {
    let content = r#"// @Filename: /node_modules/foo/index.d.ts
export const x = 0;
// @Filename: /b.ts
/// <reference types="foo" />
import { x } from "/*1*/foo";
declare module "foo" {}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyNoErrors"); // f.VerifyNoErrors(t)
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1")
}
