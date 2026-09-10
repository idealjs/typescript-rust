use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn completions_redeclare_module_as_global() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @esModuleInterop: true,
// @target: esnext
// @Filename: /myAssert.d.ts
declare function assert(value:any, message?:string):void;
export = assert;
export as namespace assert;
// @Filename: /ambient.d.ts
import assert from './myAssert';

type Assert = typeof assert;

declare global {
  const assert: Assert;
}
// @Filename: /index.ts
/// <reference path="./ambient.d.ts" />
asser/**/;"#;
    let mut s = Session::new_for_test("completionsRedeclareModuleAsGlobal", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
