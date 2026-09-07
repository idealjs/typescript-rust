use tsox_lsp::fourslash::{self, Session};

#[test]
fn this_predicate_function_quick_info() {
    let content = r#"class RoyalGuard {
    isLeader(): this is LeadGuard {
        return this instanceof LeadGuard;
    }
    isFollower(): this is FollowerGuard {
        return this instanceof FollowerGuard;
    }
}

class LeadGuard extends RoyalGuard {
    lead(): void {};
}

class FollowerGuard extends RoyalGuard {
    follow(): void {};
}

let a: RoyalGuard = new FollowerGuard();
if (a.is/*1*/Leader()) {
    a./*2*/;
}
else if (a.is/*3*/Follower()) {
    a./*4*/;
}

interface GuardInterface {
   isLeader(): this is LeadGuard;
   isFollower(): this is FollowerGuard;
}

let b: GuardInterface;
if (b.is/*5*/Leader()) {
    b./*6*/;
}
else if (b.is/*7*/Follower()) {
    b./*8*/;
}

if (((a.isLeader)())) {
    a./*9*/;
}
else if (((a).isFollower())) {
    a./*10*/;
}

if (((a["isLeader"])())) {
    a./*11*/;
}
else if (((a)["isFollower"]())) {
    a./*12*/;
}

let leader/*13*/Status = a.isLeader();
function isLeaderGuard(g: RoyalGuard) {
   return g.isLeader();
}
let checked/*14*/LeaderStatus = isLeader/*15*/Guard(a);"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(
        &mut s,
        "1",
        "(method) RoyalGuard.isLeader(): this is LeadGuard",
        "",
    );
    fourslash::verify_quick_info_at(
        &mut s,
        "3",
        "(method) RoyalGuard.isFollower(): this is FollowerGuard",
        "",
    );
    fourslash::verify_quick_info_at(
        &mut s,
        "5",
        "(method) GuardInterface.isLeader(): this is LeadGuard",
        "",
    );
    fourslash::verify_quick_info_at(
        &mut s,
        "7",
        "(method) GuardInterface.isFollower(): this is FollowerGuard",
        "",
    );
    fourslash::verify_quick_info_at(&mut s, "13", "let leaderStatus: boolean", "");
    fourslash::verify_quick_info_at(&mut s, "14", "let checkedLeaderStatus: boolean", "");
    fourslash::verify_quick_info_at(
        &mut s,
        "15",
        "function isLeaderGuard(g: RoyalGuard): g is LeadGuard",
        "",
    );
}
