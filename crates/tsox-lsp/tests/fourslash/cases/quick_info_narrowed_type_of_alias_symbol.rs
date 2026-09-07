use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_narrowed_type_of_alias_symbol() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @strict: true
// @Filename: modules.ts
export declare const someEnv: string | undefined;
// @Filename: app.ts
import { someEnv } from "./modules";
declare function isString(v: any): v is string;

if (isString(someEnv)) {
  someEnv/*1*/.charAt(0);
}"#;
    let mut s = Session::new(content);
    fourslash::go_to_file(&mut s, "app.ts");
    fourslash::go_to_marker(&mut s, "1");
    fourslash::unsupported("VerifyQuickInfoIs"); // f.VerifyQuickInfoIs(t, "(alias) const someEnv: string\nimport someEnv", "")
}
