use tsox_lsp::fourslash::{self, Session};


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
    let mut s = Session::new_for_test("completionListPrivateNamesAccessors", content);
    fourslash::verify_completions_unsorted_at(&mut s, Some("1"), &["#z", "t", "l", "y"]);
    fourslash::verify_completions_unsorted_at(&mut s, Some("2"), &["#z", "#u", "v", "k"]);
    fourslash::verify_completions_unsorted_at(&mut s, Some("3"), &["#z", "t", "l", "y"]);
    fourslash::verify_completions_exact_at(&mut s, Some("4"), &["y"]);
}
