use crate::core::mig::context::Context;
use crate::locale::Locale;

pub fn with_locale(ctx: Context, locale: Locale) -> Context { crate::fntrace::enter("with_locale"); 
    ctx.with_value(locale_context_key(), locale)
}

pub fn from_context(ctx: &Context) -> Locale { crate::fntrace::enter("from_context"); 
    ctx.value(locale_context_key()).cloned().unwrap_or_default()
}

pub fn parse(locale_str: &str) -> (Locale, bool) { crate::fntrace::enter("parse"); 
    let tag = language_tag_of(locale_str);
    let ok = !locale_str.is_empty() && is_well_formed(locale_str);
    (Locale(tag.to_string()), ok)
}

fn locale_context_key() -> &'static str { crate::fntrace::enter("locale_context_key"); 
    "locale"
}

fn language_tag_of(s: &str) -> &str { crate::fntrace::enter("language_tag_of"); 
    s
}

fn is_well_formed(s: &str) -> bool { crate::fntrace::enter("is_well_formed"); 
    let mut subtags = s.split('-');
    let Some(lang) = subtags.next() else {
        return false;
    };
    if lang.len() < 2 || lang.len() > 8 || !lang.chars().all(|c| c.is_ascii_alphabetic()) {
        return false;
    }
    for subtag in subtags {
        if subtag.is_empty() {
            return false;
        }
        if subtag.len() == 4 && subtag.chars().all(|c| c.is_ascii_alphabetic()) {
            continue;
        }
        if subtag.len() >= 5 && subtag.len() <= 8 && subtag.chars().all(|c| c.is_ascii_alphanumeric()) {
            continue;
        }
        if subtag.len() <= 4 && subtag.chars().all(|c| c.is_ascii_digit() || c.is_ascii_alphabetic()) {
            continue;
        }
        return false;
    }
    true
}
