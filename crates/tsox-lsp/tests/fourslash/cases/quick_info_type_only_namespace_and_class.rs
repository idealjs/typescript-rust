use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_type_only_namespace_and_class() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @Filename: /a.ts
export namespace ns {
  export class Box<T> {}
}
// @Filename: /b.ts
import type { ns } from './a';
let x: /*1*/ns./*2*/Box<string>;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "(alias) namespace ns\nimport ns", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "class ns.Box<T>", "")
}
