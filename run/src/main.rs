use std::collections::HashMap;
use run::Article;
use run::models::Category;
use std::env;

async fn get_articles(
    api_token: &str,
    project_id: &str,
) -> Result<Vec<Article>, Box<dyn std::error::Error>> {
    let url = format!(
        "https://api.todoist.com/rest/v2/tasks?project_id={}",
        project_id
    );

    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header("Authorization", format!("Bearer {}", api_token))
        .send()
        .await?;

    // Vérifier le status de la réponse
    if response.status().is_success() {
        println!("Success : {:?}", response.status());
    }
    if response.status().is_client_error() {
        println!("Client Error : {:?}", response.status());
        return Err(format!("Erreur client: {}", response.status()).into());
    }

    // Lire le corps de la réponse
    let mut articles: Vec<Article> = response.json().await?;
    for article in &mut articles {
        article.post_deserialize();
    }
    Ok(articles)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_token = env::var("TODOIST_API_TOKEN")
        .expect("La variable d'environnement TODOIST_API_TOKEN n'est pas définie");

    let project_id = "2332182173";

    match get_articles(&api_token, project_id).await {
        Ok(articles) => {

            let filtered_articles: Vec<Article> = exclude_put_aside_category_articles(articles);
            
            let grouped_articles = group_by_category(filtered_articles);

            // Afficher les articles groupés par catégorie
            for (category, articles) in grouped_articles.iter() {
                println!("\n{}\n{}", category.to_string(), "=".repeat(30));
                for article in articles {
                    println!("- {}\n  {}\n", article.content, article.description);
                }
            }
        }
        Err(e) => eprintln!("Erreur: {}", e),
    }

    Ok(())
}

fn group_by_category(filtered_articles: Vec<Article>) -> HashMap<Category, Vec<Article>> {
    let mut grouped_articles: HashMap<Category, Vec<Article>> = HashMap::new();

    for article in filtered_articles {
        if let Some(category) = &article.category {
            if !matches!(category, Category::PutAside) {
                grouped_articles
                    .entry(category.clone())
                    .or_insert_with(Vec::new)
                    .push(article);
            }
        }
    }
    grouped_articles
}

fn exclude_put_aside_category_articles(articles: Vec<Article>) -> Vec<Article> {
    articles
        .into_iter()
        .filter(|article| {
            !matches!(article.category, Some(Category::PutAside))
        })
        .collect()
}