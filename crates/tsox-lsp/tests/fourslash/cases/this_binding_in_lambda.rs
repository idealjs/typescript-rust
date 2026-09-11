use tsox_lsp::fourslash::{self, Session};


#[test]
fn this_binding_in_lambda() {
    let content = r#"class Greeter {
    constructor() { 
		[].forEach((anything)=>{
			console.log(th/**/is);
		});
	}
}"#;
    let mut s = Session::new_for_test("thisBindingInLambda", content);
    fourslash::verify_quick_info_at(&mut s, "", "this: this", "");
}
