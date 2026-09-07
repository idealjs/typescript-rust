use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyApplyCodeActionFromCompletion"]
#[test]
fn completions_import_46332() {
    let content = r#"// @module: esnext
// @moduleResolution: bundler
// @Filename: /node_modules/vue/package.json
{
  "name": "vue",
  "types": "dist/vue.d.ts"
}
// @Filename: /node_modules/vue/dist/vue.d.ts
export * from "@vue/runtime-dom"
// @Filename: /node_modules/@vue/runtime-dom/package.json
{
  "name": "@vue/runtime-dom",
  "types": "dist/runtime-dom.d.ts"
}
// @Filename: /node_modules/@vue/runtime-dom/dist/runtime-dom.d.ts
export * from "@vue/runtime-core";
export {}
declare module '@vue/reactivity' {
  export interface RefUnwrapBailTypes {
    runtimeDOMBailTypes: any
  }
}
// @Filename: /node_modules/@vue/runtime-core/package.json
{
  "name": "@vue/runtime-core",
  "types": "dist/runtime-core.d.ts"
}
// @Filename: /node_modules/@vue/runtime-core/dist/runtime-core.d.ts
import { ref } from '@vue/reactivity';
export { ref };
declare module '@vue/reactivity' {
  export interface RefUnwrapBailTypes {
    runtimeCoreBailTypes: any
  }
}
// @Filename: /node_modules/@vue/reactivity/package.json
{
  "name": "@vue/reactivity",
  "types": "dist/reactivity.d.ts"
}
// @Filename: /node_modules/@vue/reactivity/dist/reactivity.d.ts
export declare function ref<T = any>(): T;
// @Filename: /package.json
{
  "dependencies": {
    "vue": "*"
  }
}
// @Filename: /index.ts
import {} from "vue";
ref/**/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyApplyCodeActionFromCompletion"); // f.VerifyApplyCodeActionFromCompletion(t, new(""), &fourslash.ApplyCodeActionFromCompletionOptions{
}
