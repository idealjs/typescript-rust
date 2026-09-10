use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_module_augmentation() {
    let content = r#"// @Filename: /node_modules/foo/index.d.ts
/*1*/export type /*2*/T = number;
// @Filename: /a.ts
import * as foo from "foo";
declare module "foo" {
    export const x: /*3*/T;
}"#;
    let mut s = Session::new_for_test("findAllRefsModuleAugmentation", content);
    fourslash::verify_no_errors(&mut s, );
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
