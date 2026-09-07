use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_in_import_clause04() {
    let content = r#"// @Filename: foo.d.ts
 declare class Foo {
     static prop1(x: number): number;
     static prop1(x: string): string;
     static prop2(x: boolean): boolean;
 }
 export = Foo; /*2*/
// @Filename: app.ts
import {/*1*/} from './foo';"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyNoErrors"); // f.VerifyNoErrors(t)
    fourslash::go_to_marker(&mut s, "2");
    fourslash::unsupported("VerifyNoErrors"); // f.VerifyNoErrors(t)
}
