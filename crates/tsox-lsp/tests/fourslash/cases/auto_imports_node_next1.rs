use tsox_lsp::fourslash::{self, Session};


#[test]
fn auto_imports_node_next1() {
    let content = r#"// @module: node18
// @Filename: /node_modules/pack/package.json
{
    "name": "pack",
    "version": "1.0.0",
    "exports": {
        ".": "./main.mjs"
    }
}
// @Filename: /node_modules/pack/main.d.mts
import {} from "./unreachable.mjs";
export const fromMain = 0;
// @Filename: /node_modules/pack/unreachable.d.mts
export const fromUnreachable = 0;
// @Filename: /index.mts
import { fromMain } from "pack";
fromUnreachable/**/"#;
    let mut s = Session::new_for_test("autoImportsNodeNext1", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyImportFixAtPosition(t, []string{}, nil /*preferences*/)
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
