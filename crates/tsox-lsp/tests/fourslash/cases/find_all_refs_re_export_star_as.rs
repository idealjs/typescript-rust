use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_re_export_star_as() {
    let content = r#"// @Filename: /leafModule.ts
export const /*helloDef*/hello = () => 'Hello';
// @Filename: /exporting.ts
export * as /*leafDef*/Leaf from './leafModule';
// @Filename: /importing.ts
 import { /*leafImportDef*/Leaf } from './exporting';
 /*leafUse*/[|Leaf|]./*helloUse*/[|hello|]()"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyNoErrors"); // f.VerifyNoErrors(t)
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "helloDef", "helloUse", "leafDef", "leafImportDef", "leafUse")
}
