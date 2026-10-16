use tsox_lsp::fourslash::Session;


#[test]
fn references_for_properties_of_generic_type() {
    let content = r#"interface IFoo<T> {
    /*1*/doSomething(v: T): T;
}

var x: IFoo<string>;
x./*2*/doSomething("ss");

var y: IFoo<number>;
y./*3*/doSomething(12);"#;
    let _s = Session::new_for_test("referencesForPropertiesOfGenericType", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
