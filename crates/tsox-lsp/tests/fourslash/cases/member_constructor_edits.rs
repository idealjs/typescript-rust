use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyNoErrors"]
#[test]
fn member_constructor_edits() {
    let content = r#" namespace M {
     export class A {
		 constructor(a: string) {}
         public m(n: number) {
             return 0;
         }
         public n() {
             return this.m(0);
         }
     }
     export class B extends A {
     	constructor(a: string) {
			super(a);
		}
		/*1*/
	 }
	 var a = new A("s");
	 var b = new B("s");
 }"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyNoErrors"); // f.VerifyNoErrors(t)
    fourslash::go_to_marker(&mut s, "1");
    fourslash::insert(&mut s, "public m(n: number) { return 0; }");
    fourslash::unsupported("VerifyNoErrors"); // f.VerifyNoErrors(t)
}
