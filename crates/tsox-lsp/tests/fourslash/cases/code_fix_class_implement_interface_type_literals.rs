use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFix"]
#[test]
fn code_fix_class_implement_interface_type_literals() {
    let content = r#"type Builtin = Date | Function | Uint8Array | string | number | boolean | undefined;

export type DeepPartial<T> = T extends Builtin ? T :
    T extends Array<infer U> ? Array<DeepPartial<U>> :
        T extends ReadonlyArray<infer U> ? ReadonlyArray<DeepPartial<U>> :
            T extends {} ? { [K in keyof T]?: DeepPartial<T[K]> } : Partial<T>;

export interface Nested {
    field: string;
}

interface Foo {
    request(): DeepPartial<{ nested1: Nested; test2: Nested }>;
}
[|export class C implements Foo {}|]"#;
    let mut s = Session::new_for_test("codeFixClassImplementInterfaceTypeLiterals", content);
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
