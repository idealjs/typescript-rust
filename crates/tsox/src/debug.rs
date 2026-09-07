use std::fmt::Display;

pub trait KindString {
    fn kind_string(&self) -> String;
}

pub fn fail(reason: &str) -> ! {
    let msg = if reason.is_empty() {
        "Debug failure.".to_string()
    } else {
        format!("Debug failure. {}", reason)
    };
    panic!("{}", msg)
}

pub fn fail_bad_syntax_kind<T: KindString>(node: &T, message: Option<&str>) -> ! {
    let msg = message.unwrap_or("Unexpected node.");
    fail(&format!(
        "{}\nNode {} was unexpected.",
        msg,
        node.kind_string()
    ))
}

pub fn assert_never<T: Display>(member: &T, message: Option<&str>) -> ! {
    let msg = message.unwrap_or("Illegal value:");
    fail(&format!("{} {}", msg, member))
}

pub fn assert(value: bool, message: Option<&str>) {
    if value {
        return;
    }
    let msg = match message {
        Some(m) => format!("False expression: {}", m),
        None => "False expression.".to_string(),
    };
    fail(&msg);
}

#[cfg(test)]
mod tests;
