use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn promise_typing2() {
    let content = r#"interface IPromise<T> {
    then<U>(success?: (value: T) => IPromise<U>, error?: (error: any) => IPromise<U>, progress?: (progress: any) => void ): IPromise<U>;
    then<U>(success?: (value: T) => IPromise<U>, error?: (error: any) => U, progress?: (progress: any) => void ): IPromise<U>;
    then<U>(success?: (value: T) => U, error?: (error: any) => IPromise<U>, progress?: (progress: any) => void ): IPromise<U>;
    then<U>(success?: (value: T) => U, error?: (error: any) => U, progress?: (progress: any) => void ): IPromise<U>;
    done? <U>(success?: (value: T) => any, error?: (error: any) => any, progress?: (progress: any) => void ): void;
}
var p1: IPromise<number> = null;
p/*1*/1.then(function (x/*2*/x) { }); 
var p/*3*/2 = p1.then(function (x/*4*/x) { return "hello"; })
var p/*5*/3 = p2.then(function (x/*6*/x) {
    return x/*7*/x;
});"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "var p1: IPromise<number>", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "(parameter) xx: number", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "3", "var p2: IPromise<string>", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "4", "(parameter) xx: number", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "5", "var p3: IPromise<string>", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "6", "(parameter) xx: string", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "7", "(parameter) xx: string", "")
}
