use tsox_lsp::fourslash::Session;


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
    let _s = Session::new_for_test("goToImplementationInterface_10", content);
    // TODO: f.VerifyBaselineGoToImplementation(t, "def")
}
