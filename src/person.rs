use std::fmt;

#[derive(Debug, Clone, Copy)]
pub enum Person {
    I,
    You,
    HeSheIt,
    We,
    YouPl,
    They,
}

impl Person {
    pub fn from_str(p: &str) -> Person {
        let p = p.to_lowercase();
        return match &*p {
            "i" => Person::I,
            "you (singular)" => Person::You,
            "he/she/it" => Person::HeSheIt,
            "we" => Person::We,
            "you (plural)" => Person::YouPl,
            "they" => Person::They,
            _ => panic!("Person not matched"),
        };
    }
}

impl fmt::Display for Person {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Person::I => write!(f, "I"),
            Person::You => write!(f, "You (singular)"),
            Person::HeSheIt => write!(f, "He/she/it"),
            Person::We => write!(f, "We"),
            Person::YouPl => write!(f, "You (plural)"),
            Person::They => write!(f, "They"),
        }
    }
}
