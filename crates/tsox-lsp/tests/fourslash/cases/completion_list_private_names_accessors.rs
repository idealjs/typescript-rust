use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_private_names_accessors() {
    let content = r#"class Foo {
   get #x() { return 1 };
   set #x(value: number) { };
   y() {};
}
class Bar extends Foo {
   get #z() { return 1 };
   set #z(value: number) { };
   t() {};
   l;
   constructor() {
       this./*1*/
       class Baz {
           get #z() { return 1 };
           set #z(value: number) { };
           get #u() { return 1 };
           set #u(value: number) { };
           v() {};
           k;
           constructor() {
               this./*2*/
               new Bar()./*3*/
           }
       }
   }
}

new Foo()./*4*/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "4", &fourslash.CompletionsExpectedList{
}
