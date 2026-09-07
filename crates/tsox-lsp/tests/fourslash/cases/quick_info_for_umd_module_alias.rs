use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_for_umd_module_alias() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @Filename: 0.d.ts
export function doThing(): string;
export function doTheOtherThing(): void;
export as namespace /*0*/myLib;
// @Filename: 1.ts
/// <reference path="0.d.ts" />
/*1*/myLib.doThing();"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "0", "export namespace myLib", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "export namespace myLib", "")
}
