use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_private_names_methods() {
    let content = r#"class Foo {
   #x() {};
   y() {};
}
class Bar extends Foo {
   #z() {};
   t() {};
   constructor() {
       this./*1*/
       class Baz {
           #z() {};
           #u() {};
           v() {};
           constructor() {
               this./*2*/
               new Bar()./*3*/
           }
       }
   }
}

new Foo()./*4*/"#;
    let mut s = Session::new_for_test("completionListPrivateNamesMethods", content);
    fourslash::verify_completions_unsorted_at(&mut s, Some("1"), &["#z", "t", "y"]);
    fourslash::verify_completions_unsorted_at(&mut s, Some("2"), &["#z", "#u", "v"]);
    fourslash::verify_completions_unsorted_at(&mut s, Some("3"), &["#z", "t", "y"]);
    fourslash::verify_completions_exact_at(&mut s, Some("4"), &["y"]);
}
