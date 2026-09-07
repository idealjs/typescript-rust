use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineRename"]
#[test]
fn rename_named_import_use_aliases_for_renames() {
    let content = r#"// @Filename: /a.ts
import { /*import*/MyTypeA } from "./b";
const type: MyTypeA = { foo: "bar" };
// @Filename: /b.ts
export interface MyTypeA {
    foo: string;
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, &lsutil.UserPreferences{UseAliasesForRename: core.TSFalse}, "import")
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, &lsutil.UserPreferences{UseAliasesForRename: core.TSTrue}, "import")
}

#[ignore = "unimplemented: fourslash.VerifyBaselineRename"]
#[test]
fn rename_named_import_default_in_node_modules() {
    let content = r#"// @Filename: /index.ts
import { /*fooImport*/[|Foo|] } from "foo";
declare const f: Foo;
// @Filename: /tsconfig.json
{}
// @Filename: /node_modules/foo/package.json
{ "types": "index.d.ts" }
// @Filename: /node_modules/foo/index.d.ts
export interface Foo {
    bar: string;
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, nil /*preferences*/, "fooImport")
    fourslash::unsupported("VerifyBaselineRename"); // f.VerifyBaselineRename(t, &lsutil.UserPreferences{UseAliasesForRename: core.TSTrue}, "fooImport")
    fourslash::go_to_marker(&mut s, "fooImport");
    fourslash::unsupported("VerifyRenameFailed"); // f.VerifyRenameFailed(t, &lsutil.UserPreferences{UseAliasesForRename: core.TSFalse})
}
