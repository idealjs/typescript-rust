use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyApplyCodeActionFromCompletion"]
#[test]
fn completions_import_paths_conflict() {
    let content = r#"// @Filename: /tsconfig.json
{
    "compilerOptions": {
        "module": "esnext",
        "paths": {
          "@reduxjs/toolkit": ["src/index.ts"],
          "@internal/*": ["src/*"]
        }
    }
}
// @Filename: /src/index.ts
export { configureStore } from "./configureStore";
// @Filename: /src/configureStore.ts
export function configureStore() {}
// @Filename: /src/tests/createAsyncThunk.typetest.ts
import {} from "@reduxjs/toolkit";
/**/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyApplyCodeActionFromCompletion"); // f.VerifyApplyCodeActionFromCompletion(t, new(""), &fourslash.ApplyCodeActionFromCompletionOptions{
}
