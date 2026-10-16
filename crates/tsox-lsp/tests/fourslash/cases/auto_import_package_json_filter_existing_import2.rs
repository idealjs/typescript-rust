use tsox_lsp::fourslash::{self, Session};


#[test]
fn auto_import_package_json_filter_existing_import2() {
    let content = r#"// @lib: es5
// @module: preserve
// @Filename: /home/src/workspaces/project/node_modules/@types/react/index.d.ts
export declare function useMemo(): void;
export declare function useState(): void;
// @Filename: /home/src/workspaces/project/package.json
{}
// @Filename: /home/src/workspaces/project/index.ts
useMemo/**/"#;
    let mut s = Session::new_for_test("autoImportPackageJsonFilterExistingImport2", content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyImportFixAtPosition(t, []string{}, nil /*preferences*/)
    fourslash::go_to_bof(&mut s, );
    fourslash::insert_line(&mut s, "import { useState } from \"react\";");
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyImportFixAtPosition(t, []string{
}
