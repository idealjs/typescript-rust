use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_for_export_equals() {
    let content = r#"// @Filename: /node_modules/foo/index.d.ts
export = Foo;
declare var Foo: Foo.Static;
declare namespace Foo {
    interface Static {
        foo(): void;
    }
}
// @Filename: /a.ts
import { /**/ } from "foo";"#;
    let mut s = Session::new_for_test("completionListForExportEquals", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
