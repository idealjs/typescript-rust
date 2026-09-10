use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_for_umd_module_alias1() {
    let content = r#"// @Filename: 0.d.ts
export function doThing(): string;
export function doTheOtherThing(): void;
/*1*/export as namespace /*2*/myLib;
// @Filename: 1.ts
/// <reference path="0.d.ts" />
/*3*/myLib.doThing();"#;
    let mut s = Session::new_for_test("findAllRefsForUMDModuleAlias1", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
