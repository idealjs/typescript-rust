use tsox_lsp::fourslash::{self, Session};


#[test]
fn signature_help_constructor_call_param_properties_vs() {
    let content = r#"class Circle {
    /**
      * Initialize a circle.
      * @param  radius The radius of the circle.
      */
    constructor(private radius: number) {
    }
}
var a = new Circle(/**/"#;
    let mut s = Session::new_with_capabilities(content, None);
    // TODO: f.VerifyBaselineSignatureHelp(t)
}
