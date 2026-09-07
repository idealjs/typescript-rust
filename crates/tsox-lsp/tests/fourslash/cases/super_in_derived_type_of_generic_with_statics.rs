use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyNoErrors"]
#[test]
fn super_in_derived_type_of_generic_with_statics() {
    let content = r#"// @strict: false
namespace M {
   export class C<T extends Date> {
      static foo(): C<Date> {
          return null;
           }
     }
}
class D extends M.C<Date> {
    constructor() {
        /**/ // was an error appearing on super in editing scenarios
       }
}"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::insert(&mut s, "super();");
    fourslash::unsupported("VerifyNoErrors"); // f.VerifyNoErrors(t)
}
