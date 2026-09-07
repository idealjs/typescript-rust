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
    fourslash::verify_quick_info_at(&mut s, "0", "export namespace myLib", "");
    fourslash::verify_quick_info_at(&mut s, "1", "export namespace myLib", "");
}
