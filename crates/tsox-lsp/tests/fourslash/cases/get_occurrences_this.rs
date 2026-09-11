use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_occurrences_this() {
    let content = r#"[|this|];
[|th/**/is|];

function f() {
    this;
    this;
    () => this;
    () => {
        if (this) {
            this;
        }
        else {
            this.this;
        }
    }
    function inside() {
        this;
        (function (_) {
            this;
        })(this);
    }
}

namespace m {
    function f() {
        this;
        this;
        () => this;
        () => {
            if (this) {
                this;
            }
            else {
                this.this;
            }
        }
        function inside() {
            this;
            (function (_) {
                this;
            })(this);
        }
    }
}

class A {
    public b = this.method1;

    public method1() {
        this;
        this;
        () => this;
        () => {
            if (this) {
                this;
            }
            else {
                this.this;
            }
        }
        function inside() {
            this;
            (function (_) {
                this;
            })(this);
        }
    }

    private method2() {
        this;
        this;
        () => this;
        () => {
            if (this) {
                this;
            }
            else {
                this.this;
            }
        }
        function inside() {
            this;
            (function (_) {
                this;
            })(this);
        }
    }

    public static staticB = this.staticMethod1;

    public static staticMethod1() {
        this;
        this;
        () => this;
        () => {
            if (this) {
                this;
            }
            else {
                this.this;
            }
        }
        function inside() {
            this;
            (function (_) {
                this;
            })(this);
        }
    }

    private static staticMethod2() {
        this;
        this;
        () => this;
        () => {
            if (this) {
                this;
            }
            else {
                this.this;
            }
        }
        function inside() {
            this;
            (function (_) {
                this;
            })(this);
        }
    }
}

var x = {
    f() {
        this;
    },
    g() {
        this;
    }
}"#;
    let mut s = Session::new_for_test("getOccurrencesThis", content);
    // TODO: f.VerifyBaselineDocumentHighlights(t, nil /*preferences*/, ToAny(f.Ranges())...)
}
