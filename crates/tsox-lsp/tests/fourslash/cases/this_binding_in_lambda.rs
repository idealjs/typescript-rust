use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn this_binding_in_lambda() {
    let content = r#"class Greeter {
    constructor() { 
		[].forEach((anything)=>{
			console.log(th/**/is);
		});
	}
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "", "this: this", "")
}
