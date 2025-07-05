
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Article {
    section: i64,
    content: String,
    description: String,
    #[serde(skip_deserializing)]  // On ignore ce champ pendant la désérialisation
    #[serde(default)]
    category: Option<Category>
}

impl Default for Article {
    fn default() -> Self {
        Article {
            section: 0,
            content: String::new(),
            description: String::new(),
            category: None,
        }
    }
}

impl Article {
    // Cette méthode sera appelée après la désérialisation
    fn post_deserialize(&mut self) {
        self.category = Some(convert_to_category(&self.section));
    }
}


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

pub fn execute(data: &str) -> Result<(), serde_json::Error> {
    // Désérialiser le JSON en Vec<Article>
    let mut articles: Vec<Article> = serde_json::from_str(data)?;

    // Appliquer le post-traitement
    for article in &mut articles {
        article.post_deserialize();
    }

    // Afficher les articles
    for article in articles {
        println!("Section: {}", article.section);
        println!("Contenu: {}", article.content);
        println!("Description: {}", article.description);
        println!("Category: {:?}", article.category.expect("REASON").to_string());
        println!("---");
    }


    Ok(())
}