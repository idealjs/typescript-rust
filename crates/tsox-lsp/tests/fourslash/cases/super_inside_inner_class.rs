use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyNoSignatureHelpForMarkers"]
#[test]
fn super_inside_inner_class() {
    let content = r#"class Base {
	constructor(n: number) {
	}
}
class Derived extends Base {
	constructor() {
		class Nested {
			[super(/*1*/)] = 11111
		}
	}
}"#;
    let mut s = Session::new_for_test("superInsideInnerClass", content);
    fourslash::unsupported("VerifyNoSignatureHelpForMarkers"); // f.VerifyNoSignatureHelpForMarkers(t, "1")
}
