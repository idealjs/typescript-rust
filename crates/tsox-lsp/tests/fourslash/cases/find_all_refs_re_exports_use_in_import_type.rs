use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_re_exports_use_in_import_type() {
    let content = r#"// @Filename: /foo/types/types.ts
[|export type /*full0*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 0 |}Full|] = { prop: string; };|]
// @Filename: /foo/types/index.ts
[|import * as /*foo0*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 2 |}foo|] from './types';|]
[|export { /*foo1*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 4 |}foo|] };|]
// @Filename: /app.ts
[|import { /*foo2*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 6 |}foo|] } from './foo/types';|]
export type fullType = /*foo3*/[|foo|]./*full1*/[|Full|];
type namespaceImport = typeof import('./foo/types');
type fullType2 = import('./foo/types')./*foo4*/[|foo|]./*full2*/[|Full|];"#;
    let mut s = Session::new(content);
    fourslash::verify_no_errors(&mut s);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "full0", "full1", "full2", "foo0", "foo1", "foo2", "foo3", "foo
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[1], f.Ranges()[9], f.Ranges()[11])
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[3])
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[5], f.Ranges()[10])
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[7], f.Ranges()[8])
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, &lsutil.UserPreferences{UseAliasesForRename: core.TSFalse}, f.Ranges()[7],
}
