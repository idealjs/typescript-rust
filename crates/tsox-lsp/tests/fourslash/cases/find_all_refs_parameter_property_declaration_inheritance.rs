use tsox_lsp::fourslash::{self, Session};


#[test]
fn find_all_refs_parameter_property_declaration_inheritance() {
    let content = r#"class C {
	constructor(public /*0*/x: string) {
		/*1*/x;
	}
}
class D extends C {
	constructor(public /*2*/x: string) {
		super(/*3*/x);
	}
}"#;
    let mut s = Session::new_for_test("findAllRefsParameterPropertyDeclaration_inheritance", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "0", "1", "2", "3")
}
