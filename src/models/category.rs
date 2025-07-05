use serde::{Deserialize, Serialize};

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

pub fn convert_to_category(section: &i64) -> Category {
    match section {
        181074705 => Category::Youtube,
        179438112 => Category::Articles,
        181074629 => Category::Tools,
        184011119 => Category::Podcast,
        184719314 => Category::Livre,
        _ => Category::PutAside
    }
}