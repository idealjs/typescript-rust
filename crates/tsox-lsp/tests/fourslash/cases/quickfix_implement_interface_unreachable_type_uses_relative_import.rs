use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn quickfix_implement_interface_unreachable_type_uses_relative_import() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @Filename: class.ts
export class Class { }
// @Filename: interface.ts
import { Class } from './class';

export interface Foo {
    x: Class;
}
// @Filename: index.ts
import { Foo } from './interface';

class /*1*/X implements Foo {}"#;
    let mut s = Session::new_for_test("quickfixImplementInterfaceUnreachableTypeUsesRelativeImport", content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
