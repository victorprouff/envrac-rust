use run::Article;
use run::models::Category;
use std::collections::HashMap;
use std::env;
use chrono::{Local, Datelike};

const MOIS: [&str; 12] = ["Janvier", "Février", "Mars", "Avril", "Mai", "Juin",
    "Juillet", "Août", "Septembre", "Octobre", "Novembre", "Décembre"];

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

            let now = Local::now();
            let day_letter = format!("{} {}",
                                     now.format("%d").to_string(),
                                     MOIS[now.month0() as usize]
            );

            let mut content_blog = format!("---
title: \"[En Vrac] - {dayLetter}\"
description: \"En vrac du {dayLetter}. Mes découvertes, articles, vidéos et écoute qui m'ont intéressé et que je veux partager.\"
summary: \"En vrac du {dayLetter}. Mes découvertes, articles, vidéos et écoute qui m'ont intéressé et que je veux partager.\"
date: {year}-{month}-{day}T05:00:03+01:00
categories: [ \"En vrac\" ]
draft: false
---

Hello ! 😊

Comme chaque semaine, vous pouvez retrouver ici des liens d’articles de vidéos ou de podcast que j’ai découvert au fil de ma veille quotidienne et que j’aimerais partager avec vous. 😀

Les deux derniers EnVrac :
  - [[En Vrac] - {lastArticle1date}](https://blog.victorprouff.fr/en-vracs/${lastArticle1name}/)
  - [[En Vrac] - {lastArticle2date}](https://blog.victorprouff.fr/en-vracs/${lastArticle2name}/)",
        dayLetter = day_letter,
        year = now.year(),
        month = now.format("%m").to_string(),
        day = now.format("%d").to_string(),
        lastArticle1date = "lastArticle1date",
        lastArticle1name = "lastArticle1name",
        lastArticle2date = "lastArticle2date",
        lastArticle2name = "lastArticle2name");

            // Afficher les articles groupés par catégorie
            for (category, articles) in grouped_articles.iter() {
                content_blog.push_str(&format!("\n\n## {}\n", category.to_string()));
                for article in articles {
                    content_blog.push_str(&format!("- {}", article.content));
                    if !article.description.is_empty() {
                        content_blog.push_str(&format!(" - {}\n", article.description));
                    }
                    else {
                        content_blog.push_str(&"\n".to_string());
                    }
                }
            }

            println!("{}",content_blog);
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
        .filter(|article| !matches!(article.category, Some(Category::PutAside)))
        .collect()
}
