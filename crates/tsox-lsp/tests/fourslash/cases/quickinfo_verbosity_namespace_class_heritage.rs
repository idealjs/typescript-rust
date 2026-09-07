use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineHoverWithVerbosity"]
#[test]
fn quickinfo_verbosity_namespace_class_heritage() {
    let content = r#"
declare class Base {
    id: number;
}

declare namespace Shapes/*1*/ {
    class Circle extends Base {
        radius: number;
    }
    class Square extends Base {
        side: number;
    }
    interface Drawable {
        draw(): void;
    }
}
"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineHoverWithVerbosity"); // f.VerifyBaselineHoverWithVerbosity(t, map[string][]int{"1": {0, 1}})
}
