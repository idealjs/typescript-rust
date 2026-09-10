use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToImplementation"]
#[test]
fn go_to_implementation_interface_10() {
    let content = r#"// @Filename: /a.ts
interface /*def*/A {
	foo: boolean;
}
interface [|B|] extends A {
	bar: boolean;
}
export class [|C|] implements B {
	foo = true;
	bar = true;
}
export class [|D|] extends C { }"#;
    let mut s = Session::new_for_test("goToImplementationInterface_10", content);
    fourslash::unsupported("VerifyBaselineGoToImplementation"); // f.VerifyBaselineGoToImplementation(t, "def")
}
