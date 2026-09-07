use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: f.MarkTestAsStradaServer()"]
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
    let mut s = Session::new(content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{}, nil /*preferences*/)
    fourslash::unsupported("GoToBOF"); // f.GoToBOF(t)
    fourslash::unsupported("InsertLine"); // f.InsertLine(t, "import { useState } from \"react\";")
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyImportFixAtPosition"); // f.VerifyImportFixAtPosition(t, []string{
}
