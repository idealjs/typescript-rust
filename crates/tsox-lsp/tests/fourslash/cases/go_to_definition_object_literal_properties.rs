use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_object_literal_properties() {
    let content = r#"var o = {
    /*valueDefinition*/value: 0,
    get /*getterDefinition*/getter() {return 0 },
    set /*setterDefinition*/setter(v: number) { },
    /*methodDefinition*/method: () => { },
    /*es6StyleMethodDefinition*/es6StyleMethod() { }
};

o./*valueReference*/value;
o./*getterReference*/getter;
o./*setterReference*/setter;
o./*methodReference*/method;
o./*es6StyleMethodReference*/es6StyleMethod;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, false, "valueReference", "getterReference", "setterReference", "me
}
