use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn references_for_statics_and_members_with_same_names() {
    let content = r#"namespace FindRef4 {
	namespace MixedStaticsClassTest {
		export class Foo {
			/*1*/bar: Foo;
			/*2*/static /*3*/bar: Foo;

			/*4*/public /*5*/foo(): void {
			}
			/*6*/public static /*7*/foo(): void {
			}
		}
	}

	function test() {
		// instance function
		var x = new MixedStaticsClassTest.Foo();
		x./*8*/foo();
		x./*9*/bar;

		// static function
		MixedStaticsClassTest.Foo./*10*/foo();
		MixedStaticsClassTest.Foo./*11*/bar;
	}
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11")
}
