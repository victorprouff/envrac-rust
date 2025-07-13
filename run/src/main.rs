use run::Article;
use std::env;

async fn get_articles(api_token: &str, project_id: &str) -> Result<Vec<Article>, Box<dyn std::error::Error>> {

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
    let articles: Vec<Article> = response.json().await?;

    Ok(articles)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let api_token = env::var("TODOIST_API_TOKEN")
        .expect("La variable d'environnement TODOIST_API_TOKEN n'est pas définie");

    let project_id = "2332182173";

    match get_articles(&api_token, project_id).await {
        Ok(mut articles) => {
            for article in &mut articles {
                article.post_deserialize();
            }

            for article in articles {
                println!("Article : {}, {} - Catégorie: {:?}",
                         article.content,
                         article.description,
                         article.category.unwrap().to_string());
            }
        }
        Err(e) => eprintln!("Erreur: {}", e),
    }

    Ok(())
}
