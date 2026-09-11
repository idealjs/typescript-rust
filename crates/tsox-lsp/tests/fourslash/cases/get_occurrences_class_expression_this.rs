use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_occurrences_class_expression_this() {
    let content = r#"var x = class C {
    public x;
    public y;
    public z;
    constructor() {
        [|this|];
        [|this|].x;
        [|this|].y;
        [|this|].z;
    }
    foo() {
        [|this|];
        () => [|this|];
        () => {
            if ([|this|]) {
                [|this|];
            }
        }
        function inside() {
            this;
            (function (_) {
                this;
            })(this);
        }
        return [|this|].x;
    }

    static bar() {
        this;
        () => this;
        () => {
            if (this) {
                this;
            }
        }
        function inside() {
            this;
            (function (_) {
                this;
            })(this);
        }
    }
}"#;
    let mut s = Session::new_for_test("getOccurrencesClassExpressionThis", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
