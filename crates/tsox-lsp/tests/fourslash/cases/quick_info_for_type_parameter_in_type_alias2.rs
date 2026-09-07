use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_for_type_parameter_in_type_alias2() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"type Call<AA> = { (): A/*1*/A };
type Index<AA> = {[foo: string]: A/*2*/A};
type GenericMethod<AA> = { method<BB>(): A/*3*/A & B/*4*/B }
type Nesting<TT> = { method<UU>(): new <WW>() => T/*5*/T & U/*6*/U & W/*7*/W };"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(&mut s, "1", "(type parameter) AA in type Call<AA>", "");
    fourslash::verify_quick_info_at(&mut s, "2", "(type parameter) AA in type Index<AA>", "");
    fourslash::verify_quick_info_at(
        &mut s,
        "3",
        "(type parameter) AA in type GenericMethod<AA>",
        "",
    );
    fourslash::verify_quick_info_at(
        &mut s,
        "4",
        "(type parameter) BB in method<BB>(): AA & BB",
        "",
    );
    fourslash::verify_quick_info_at(&mut s, "5", "(type parameter) TT in type Nesting<TT>", "");
    fourslash::verify_quick_info_at(
        &mut s,
        "6",
        "(type parameter) UU in method<UU>(): new <WW>() => TT & UU & WW",
        "",
    );
    fourslash::verify_quick_info_at(
        &mut s,
        "7",
        "(type parameter) WW in <WW>(): TT & UU & WW",
        "",
    );
}
