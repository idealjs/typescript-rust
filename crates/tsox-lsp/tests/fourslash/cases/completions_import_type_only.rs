use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyApplyCodeActionFromCompletion"]
#[test]
fn completions_import_type_only() {
    let content = r#"// @target: esnext
// @moduleResolution: bundler
// @Filename: /a.ts
export class A {}
export class B {}
// @Filename: /b.ts
import type { A } from './a';
const b: B/**/"#;
    let mut s = Session::new_for_test("completionsImport_typeOnly", content);
    fourslash::go_to_file(&mut s, "/b.ts");
    fourslash::unsupported("VerifyApplyCodeActionFromCompletion"); // f.VerifyApplyCodeActionFromCompletion(t, new(""), &fourslash.ApplyCodeActionFromCompletionOptions{
}
