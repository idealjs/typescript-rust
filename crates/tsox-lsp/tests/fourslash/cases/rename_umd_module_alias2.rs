use tsox_lsp::fourslash::{self, Session};


#[test]
fn rename_umd_module_alias2() {
    let content = r#"// @Filename: 0.d.ts
export function doThing(): string;
export function doTheOtherThing(): void;
export as namespace /**/[|myLib|];
// @Filename: 1.ts
/// <reference path="0.d.ts" />
myLib.doThing();"#;
    let mut s = Session::new_for_test("renameUMDModuleAlias2", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyRenameSucceeded(t, nil /*preferences*/)
}
