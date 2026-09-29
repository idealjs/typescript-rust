use tsox_lsp::fourslash::Session;


#[test]
fn navigation_bar_property_declarations() {
    let content = r#"class A {
    public A1 = class {
        public x = 1;
        private y() {}
        protected z() {}
    }

    public A2 = {
        x: 1,
        y() {},
        z() {}
    }

    public A3 = function () {}
    public A4 = () => {}
    public A5 = 1;
    public A6 = "A6";

    public ["A7"] = class {
        public x = 1;
        private y() {}
        protected z() {}
    }

    public [1] = {
        x: 1,
        y() {},
        z() {}
    }

    public [1 + 1] = 1;
}"#;
    let _s = Session::new_for_test("navigationBarPropertyDeclarations", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
