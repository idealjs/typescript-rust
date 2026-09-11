use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_literal_type_in_union_or_intersection_type() {
    let content = r#"type NumberAndString = {
    a: number
} & {
    b: string
};

type NumberOrString = {
    a: number
} | {
    b: string
};

type Complexed =
    Foo &
    Bar |
    Baz;"#;
    let mut s = Session::new_for_test("formatLiteralTypeInUnionOrIntersectionType", content);
    // TODO: f.FormatDocument(t, "")
    fourslash::verify_current_file_content(&mut s, r#"type NumberAndString = {
    a: number
} & {
    b: string
};

type NumberOrString = {
    a: number
} | {
    b: string
};

type Complexed =
    Foo &
    Bar |
    Baz;"#);
}
