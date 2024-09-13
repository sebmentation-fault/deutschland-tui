use std::fmt;

#[derive(Debug, Clone, Copy)]
pub enum Verb {
    Aufwachen,
    Duschen,
    Essen,
    Gehen,
    Haben,
    Machen,
    Helfen,
    Schlafen,
    Skifahren,
    Treffen,
    Trinken,
}

impl Verb {
    pub fn from_str(v: &str) -> Verb {
        let v = v.to_lowercase();
        return match &*v {
            "aufwachen" => Verb::Aufwachen,
            "duschen" => Verb::Duschen,
            "essen" => Verb::Essen,
            "gehen" => Verb::Gehen,
            "haben" => Verb::Haben,
            "helfen" => Verb::Helfen,
            "machen" => Verb::Machen,
            "schlafen" => Verb::Schlafen,
            "skifahren" => Verb::Skifahren,
            "treffen" => Verb::Treffen,
            "trinken" => Verb::Trinken,
            _ => panic!("Verb not matched"),
        };
    }
}

impl fmt::Display for Verb {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Verb::Aufwachen => write!(f, "Aufwachen"),
            Verb::Duschen => write!(f, "Duschen"),
            Verb::Essen => write!(f, "Essen"),
            Verb::Gehen => write!(f, "Gehen"),
            Verb::Haben => write!(f, "Haben"),
            Verb::Helfen => write!(f, "Helfen"),
            Verb::Machen => write!(f, "Machen"),
            Verb::Schlafen => write!(f, "Schlafen"),
            Verb::Skifahren => write!(f, "Skifahren"),
            Verb::Treffen => write!(f, "Treffen"),
            Verb::Trinken => write!(f, "Trinken"),
        }
    }
}
