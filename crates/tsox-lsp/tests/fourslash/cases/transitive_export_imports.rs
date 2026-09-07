use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn transitive_export_imports() {
    let content = r#"// @module: commonjs
// @Filename: a.ts
[|class /*1*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 0 |}A|] {
}|]
[|export = [|{| "contextRangeIndex": 2 |}A|];|]
// @Filename: b.ts
[|export import /*2*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 4 |}b|] = require('./a');|]
// @Filename: c.ts
[|import /*3*/[|{| "isWriteAccess": true, "isDefinition": true, "contextRangeIndex": 6 |}b|] = require('./b');|]
var a = new /*4*/[|b|]./**/[|b|]();"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyQuickInfoExists"); // f.VerifyQuickInfoExists(t)
    fourslash::verify_no_errors(&mut s);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4")
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[1], f.Ranges()[3])
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[5], f.Ranges()[9])
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, f.Ranges()[7], f.Ranges()[8])
}
