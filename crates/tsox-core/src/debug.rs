pub(crate) use std::fmt::Display;

pub trait KindString {
    fn kind_string(&self) -> String;
}

pub fn fail(reason: &str) -> ! { crate::fntrace::enter("fail"); 
    let msg = if reason.is_empty() {
        "Debug failure.".to_string()
    } else {
        format!("Debug failure. {}", reason)
    };
    panic!("{}", msg)
}

pub fn fail_bad_syntax_kind<T: KindString>(node: &T, message: Option<&str>) -> ! { crate::fntrace::enter("fail_bad_syntax_kind"); 
    let msg = message.unwrap_or("Unexpected node.");
    fail(&format!(
        "{}\nNode {} was unexpected.",
        msg,
        node.kind_string()
    ))
}

pub fn assert_never<T: Display>(member: &T, message: Option<&str>) -> ! { crate::fntrace::enter("assert_never"); 
    let msg = message.unwrap_or("Illegal value:");
    fail(&format!("{} {}", msg, member))
}

pub fn assert(value: bool, message: Option<&str>) { crate::fntrace::enter("assert"); 
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
pub(crate) mod tests;
