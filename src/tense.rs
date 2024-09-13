use std::fmt;

#[derive(Debug, Clone, Copy)]
pub enum Tense {
    Present,
    PerfectPresent,
    Past,
    PerfectPast,
    Future,
    PerfectFuture,
    SubjectiveI,
    SubjectiveII,
}

impl Tense {
    pub fn from_str(t: &str) -> Tense {
        let t = t.to_lowercase();
        match &*t {
            "present" => Tense::Present,
            "perfectpresent" => Tense::PerfectPresent,
            "past" => Tense::Past,
            "perfectpast" => Tense::PerfectPast,
            "future" => Tense::Future,
            "perfectfuture" => Tense::PerfectFuture,
            "subjectivei" => Tense::SubjectiveI,
            "subjectiveii" => Tense::SubjectiveII,
            _ => panic!("Tense not matched: {}", t),
        }
    }
}

impl fmt::Display for Tense {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Tense::Present => write!(f, "Present"),
            Tense::PerfectPresent => write!(f, "Perfect Present"),
            Tense::Past => write!(f, "Past"),
            Tense::PerfectPast => write!(f, "Perfect Past"),
            Tense::Future => write!(f, "Future"),
            Tense::PerfectFuture => write!(f, "Perfect Future"),
            Tense::SubjectiveI => write!(f, "Subjective I"),
            Tense::SubjectiveII => write!(f, "Subjective II"),
        }
    }
}
