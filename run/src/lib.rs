
// Déclare le module models
pub mod models;
// Re-exporte Article pour un accès plus facile
pub use models::Article;

/*
pub fn execute(data: &str) -> Result<(), serde_json::Error> {
    // Désérialiser le JSON en Vec<Article>
    let mut articles: Vec<Article> = serde_json::from_str(data)?;

    // Appliquer le post-traitement
    for article in &mut articles {
        article.post_deserialize();
    }

    // Afficher les articles
    for article in articles {
        println!("Contenu: {}", article.content);
        println!("Description: {}", article.description);
        println!("Category: {:?}", article.category.expect("REASON").to_string());
        println!("---");
    }

    Ok(())
}*/