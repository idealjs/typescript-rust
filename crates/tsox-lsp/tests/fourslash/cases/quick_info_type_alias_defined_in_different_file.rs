use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_type_alias_defined_in_different_file() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @Filename: /a.ts
export type X = { x: number };
export function f(x: X): void {}
// @Filename: /b.ts
import { f } from "./a";
/**/f({ x: 1 });"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "", "(alias) f(x: X): void\nimport f", "")
}
