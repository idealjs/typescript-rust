use tsox_lsp::fourslash::{self, Session};


#[test]
fn promise_typing1() {
    let content = r#"interface IPromise<T> {
    then<U>(success: (value: T) => IPromise<U>, error?: (error: any) => IPromise<U>, progress?: (progress: any) => void ): IPromise<U>;
    then<U>(success: (value: T) => IPromise<U>, error?: (error: any) => U, progress?: (progress: any) => void ): IPromise<U>;
    then<U>(success: (value: T) => U, error?: (error: any) => IPromise<U>, progress?: (progress: any) => void ): IPromise<U>;
    then<U>(success: (value: T) => U, error?: (error: any) => U, progress?: (progress: any) => void ): IPromise<U>;
    done? <U>(success: (value: T) => any, error?: (error: any) => any, progress?: (progress: any) => void ): void;
}
var p1: IPromise<string>;
var p/*1*/2 = p1.then(function (x/*2*/x) {
    return xx;
});
p2.then(function (x/*3*/x) {
} );"#;
    let mut s = Session::new_for_test("promiseTyping1", content);
    fourslash::verify_quick_info_at(&mut s, "1", "var p2: IPromise<string>", "");
    fourslash::verify_quick_info_at(&mut s, "2", "(parameter) xx: string", "");
    fourslash::verify_quick_info_at(&mut s, "3", "(parameter) xx: string", "");
}
