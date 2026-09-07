use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.FormatDocument"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::verify_current_file_content(
        &mut s,
        r#"type NumberAndString = {
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
    Baz;"#,
    );
}
