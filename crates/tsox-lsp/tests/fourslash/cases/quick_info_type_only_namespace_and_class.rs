use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_type_only_namespace_and_class() {
    let content = r#"// @Filename: /a.ts
export namespace ns {
  export class Box<T> {}
}
// @Filename: /b.ts
import type { ns } from './a';
let x: /*1*/ns./*2*/Box<string>;"#;
    let mut s = Session::new_for_test("quickInfoTypeOnlyNamespaceAndClass", content);
    fourslash::verify_quick_info_at(&mut s, "1", "(alias) namespace ns\nimport ns", "");
    fourslash::verify_quick_info_at(&mut s, "2", "class ns.Box<T>", "");
}
