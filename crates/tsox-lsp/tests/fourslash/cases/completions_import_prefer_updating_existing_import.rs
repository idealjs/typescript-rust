use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_import_prefer_updating_existing_import() {
    let content = r#"// @lib: es5
// @module: commonjs
// @Filename: /deep/module/why/you/want/this/path.ts
export const x = 0;
export const y = 1;
// @Filename: /nice/reexport.ts
import { x, y } from "../deep/module/why/you/want/this/path";
export { x, y };
// @Filename: /index.ts
import { x } from "./deep/module/why/you/want/this/path";

y/**/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
