use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: }"]
#[test]
fn generics_formatting() {
    let content = r#"/*inClassDeclaration*/class Foo   <    T1   ,  T2    >  {
/*inMethodDeclaration*/    public method    <   T3,    T4   >   ( a: T1,   b: Array    < T4 > ):   Map < T1  ,   T2, Array < T3    >    > {
    }
}
/*typeArguments*/var foo = new Foo   <  number, Array <   number  >   >  (  );
/*typeArgumentsWithTypeLiterals*/foo = new Foo  <  {   bar  :  number }, Array   < {   baz :  string   }  >  >  (  );

interface IFoo {
/*inNewSignature*/new < T  > ( a: T);
/*inOptionalMethodSignature*/op?< T , M > (a: T, b : M );
}

foo()<number, string, T >();
(a + b)<number, string, T >();

/*inFunctionDeclaration*/function bar <T> () {
/*inClassExpression*/    return class  <  T2 > {
    }
}
/*expressionWithTypeArguments*/class A < T > extends bar <  T >( )  <  T > {
}"#;
    let mut s = Session::new_for_test("genericsFormatting", content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::go_to_marker(&mut s, "inClassDeclaration");
    fourslash::verify_current_line_content(&mut s, r#"class Foo<T1, T2> {"#);
    fourslash::go_to_marker(&mut s, "inMethodDeclaration");
    fourslash::verify_current_line_content(&mut s, r#"    public method<T3, T4>(a: T1, b: Array<T4>): Map<T1, T2, Array<T3>> {"#);
    fourslash::go_to_marker(&mut s, "typeArguments");
    fourslash::verify_current_line_content(&mut s, r#"var foo = new Foo<number, Array<number>>();"#);
    fourslash::go_to_marker(&mut s, "typeArgumentsWithTypeLiterals");
    fourslash::verify_current_line_content(&mut s, r#"foo = new Foo<{ bar: number }, Array<{ baz: string }>>();"#);
    fourslash::go_to_marker(&mut s, "inNewSignature");
    fourslash::verify_current_line_content(&mut s, r#"    new <T>(a: T);"#);
    fourslash::go_to_marker(&mut s, "inOptionalMethodSignature");
    fourslash::verify_current_line_content(&mut s, r#"    op?<T, M>(a: T, b: M);"#);
    fourslash::go_to_marker(&mut s, "inFunctionDeclaration");
    fourslash::verify_current_line_content(&mut s, r#"function bar<T>() {"#);
    fourslash::go_to_marker(&mut s, "inClassExpression");
    fourslash::verify_current_line_content(&mut s, r#"    return class <T2> {"#);
    fourslash::go_to_marker(&mut s, "expressionWithTypeArguments");
    fourslash::verify_current_line_content(&mut s, r#"class A<T> extends bar<T>()<T> {"#);
    // TODO: }
}
