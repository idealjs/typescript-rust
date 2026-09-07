use super::*;

#[derive(Clone, Debug)]
pub struct Glob {
    pub(super) elems: Vec<Element>,
}

impl Glob {
    pub fn parse(pattern: &str) -> Result<Glob, String> {
        let (g, _rest) = parse::parse_inner(pattern, false)?;
        Ok(g)
    }

    pub fn is_match(&self, input: &str) -> bool {
        matcher::match_elements(&self.elems, input)
    }
}

impl std::fmt::Display for Glob {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for e in &self.elems {
            match e {
                Element::Slash => write!(f, "/")?,
                Element::Literal(s) => write!(f, "{}", s)?,
                Element::Star => write!(f, "*")?,
                Element::AnyChar => write!(f, "?")?,
                Element::StarStar => write!(f, "**")?,
                Element::Group(gs) => {
                    write!(f, "{{")?;
                    for (i, g) in gs.iter().enumerate() {
                        if i > 0 {
                            write!(f, ",")?;
                        }
                        write!(f, "{}", g)?;
                    }
                    write!(f, "}}")?;
                }
                Element::CharRange { negate, low, high } => {
                    write!(f, "[")?;
                    if *negate {
                        write!(f, "!")?;
                    }
                    write!(f, "{}-{}", low, high)?;
                    write!(f, "]")?;
                }
            }
        }
        Ok(())
    }
}
