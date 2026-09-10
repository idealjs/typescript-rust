use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn this_predicate_function_quick_info01() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"class FileSystemObject {
    /*1*/isFile(): this is Item {
        return this instanceof Item;
    }
    /*2*/isDirectory(): this is Directory {
        return this instanceof Directory;
    }
    /*3*/isNetworked(): this is (Networked & this) {
       return !!(this as Networked).host;
    }
    constructor(public path: string) {}
}

class Item extends FileSystemObject {
    constructor(path: string, public content: string) { super(path); }
}
class Directory extends FileSystemObject {
    children: FileSystemObject[];
}
interface Networked {
    host: string;
}

const obj: FileSystemObject = new Item("/foo", "");
if (obj.isFile/*4*/()) {
    obj.;
    if (obj.isNetworked/*5*/()) {
        obj.;
    }
}
if (obj.isDirectory/*6*/()) {
    obj.;
    if (obj.isNetworked/*7*/()) {
        obj.;
    }
}
if (obj.isNetworked/*8*/()) {
    obj.;
}"#;
    let mut s = Session::new_for_test("thisPredicateFunctionQuickInfo01", content);
    fourslash::verify_quick_info_at(&mut s, "1", "(method) FileSystemObject.isFile(): this is Item", "");
    fourslash::verify_quick_info_at(&mut s, "2", "(method) FileSystemObject.isDirectory(): this is Directory", "");
    fourslash::verify_quick_info_at(&mut s, "3", "(method) FileSystemObject.isNetworked(): this is (Networked & this)", "");
    fourslash::verify_quick_info_at(&mut s, "4", "(method) FileSystemObject.isFile(): this is Item", "");
    fourslash::verify_quick_info_at(&mut s, "5", "(method) FileSystemObject.isNetworked(): this is (Networked & Item)", "");
    fourslash::verify_quick_info_at(&mut s, "6", "(method) FileSystemObject.isDirectory(): this is Directory", "");
    fourslash::verify_quick_info_at(&mut s, "7", "(method) FileSystemObject.isNetworked(): this is (Networked & Directory)", "");
    fourslash::verify_quick_info_at(&mut s, "8", "(method) FileSystemObject.isNetworked(): this is (Networked & FileSystemObject)", "");
}
