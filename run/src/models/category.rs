use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Serialize, Deserialize, Clone)]
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

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.to_string())
    }
}

pub fn convert_to_category(section: &str) -> Category {
    match section {
        "181074705" => Category::Youtube,
        "179438112" => Category::Articles,
        "181074629" => Category::Tools,
        "184011119" => Category::Podcast,
        "184719314" => Category::Livre,
        _ => Category::PutAside
    }
}