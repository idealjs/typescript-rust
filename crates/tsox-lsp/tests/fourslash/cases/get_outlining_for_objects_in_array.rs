use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_outlining_for_objects_in_array() {
    let content = r#"const x =[| [
    [|{ a: 0 }|],
    [|{ b: 1 }|],
    [|{ c: 2 }|]
]|];

const y =[| [
    [|{
        a: 0
    }|],
    [|{
        b: 1
    }|],
    [|{
        c: 2
    }|]
]|];

const w =[| [
    [|[ 0 ]|],
    [|[ 1 ]|],
    [|[ 2 ]|]
]|];

const z =[| [
    [|[
        0
    ]|],
    [|[
        1
    ]|],
    [|[
        2
    ]|]
]|];

const z =[| [
    [|[
        [|{ hello: 0 }|]
    ]|],
    [|[
        [|{ hello: 3 }|]
    ]|],
    [|[
        [|{ hello: 5 }|],
        [|{ hello: 7 }|]
    ]|]
]|];"#;
    let mut s = Session::new_for_test("getOutliningForObjectsInArray", content);
    // TODO: f.VerifyOutliningSpans(t)
}
