use serde::{Deserialize, Serialize};
use std::fmt;
use std::hash::{Hash, Hasher};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Category {
    Youtube,
    Articles,
    Tools,
    Podcast,
    Livre,
    PutAside
}

impl Category {
    pub fn to_string(&self) -> &'static str {
        match self {
            Category::Youtube => "🎞️ Youtube",
            Category::Articles => "📖 Articles",
            Category::Tools => "🛠️ Tools",
            Category::Podcast => "🎧 Podcasts",
            Category::Livre => "📚 Livres",
            Category::PutAside => "Autre",
        }
    }
}

impl Eq for Category {} // Comme nous avons déjà PartialEq, cette implémentation vide suffit

impl Hash for Category {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // On utilise discriminant pour obtenir une valeur unique pour chaque variante
        std::mem::discriminant(self).hash(state);
    }
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.to_string())
    }
}

pub fn convert_to_category(section: &str) -> Category {
    match section {
        "6X8mJ9WhpJFPrfmm" => Category::Youtube,   // Vidéos
        "6X5QxC5v5w5xp7Mm" => Category::Articles,  // Articles
        "6X8mJ5rj5jH9mp6m" => Category::Tools,     // Tools
        "6XMH39GP3Rc82GfF" => Category::Podcast,   // Podcasts
        "6XQ22GHFMvw4gxVF" => Category::Livre,     // Livres
        _ => Category::PutAside                     // Semaine prochaine, On garde de côté, etc.
    }
}