use tsox_lsp::fourslash::Session;


#[test]
fn go_to_type_with_tuple_types1() {
    let content = r#"
export let x/*1*/: [number, number] = [1, 2];

type DoubleTupleTrouble<T> = [T, T];

export let y/*2*/: DoubleTupleTrouble<number> = [1, 2];
"#;
    let _s = Session::new_for_test("goToTypeWithTupleTypes1", content);
    // TODO: f.VerifyBaselineGoToTypeDefinition(t, f.MarkerNames()...)
}
