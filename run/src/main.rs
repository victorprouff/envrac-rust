use run::Task;
use run::models::{Author, Category, Committer, GithubRequest};
use std::collections::HashMap;
use std::{env};
use base64::Engine;
use base64::engine::general_purpose;
use chrono::{Local, Datelike, DateTime, NaiveDate};
use reqwest::header::USER_AGENT;
use serde::Deserialize;

const MOIS: [&str; 12] = ["Janvier", "Février", "Mars", "Avril", "Mai", "Juin",
    "Juillet", "Août", "Septembre", "Octobre", "Novembre", "Décembre"];

async fn get_todoist_tasks(
    api_token: &str,
    project_id: &str,
) -> Result<Vec<Task>, Box<dyn std::error::Error>> {
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
    let mut articles: Vec<Task> = response.json().await?;
    for article in &mut articles {
        article.post_deserialize();
    }
    Ok(articles)
}

#[derive(Deserialize, Debug)]
struct Content {
    name: String,
    #[serde(skip_deserializing)]  // On ignore ce champ pendant la désérialisation
    #[serde(default)]
    date: String,
}

async fn get_last_articles_blog(api_token: &str, user_agent: &str) -> Result<Vec<Content>, Box<dyn std::error::Error>> {
    let url = "https://api.github.com/repos/victorprouff/blog-hugo/contents/content/en-vracs";

    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header("Accept", "application/vnd.github+json")
        .header("Authorization", format!("Bearer {}", api_token))
        .header(USER_AGENT, user_agent) // gh api requires a user-agent header
        .header("X-GitHub-Api-Version", "2022-11-28")
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
    let mut content: Vec<Content> = response.json().await?;

    content.sort_by(|a, b| b.name.cmp(&a.name));
    content.truncate(2);

    for content in &mut content {
        content.name = content.name.replace(".md", "").to_lowercase();
        content.date = content.name[..10].to_string();
    }
    Ok(content)
}

async fn push_new_article_blog(api_token: &str, user_agent: &str, content: &str, commit_message: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let file_name = format!("{}.md", Local::now().format("%Y-%m-%d-EnVrac"));
    let base_url = "https://api.github.com/repos/victorprouff/blog-hugo/contents/content/en-vracs";
    let file_url = format!("{}/{}", base_url, file_name);

    let encoded_content = general_purpose::STANDARD.encode(content);

    let body = GithubRequest {
        message: commit_message.to_string(),
        committer: Committer {
            name: "Victor Prouff".to_string(),
            email: "victorprouff@outlook.fr".to_string(),
        },
        author: Author {
            name: "Victor Prouff".to_string(),
            email: "victorprouff@outlook.fr".to_string(),
        },
        content: encoded_content,
        branch: "main".to_string()
    };

    let client = reqwest::Client::new();
    let response = client
        .put(file_url)
        .header("Accept", "application/vnd.github+json")
        .header("Authorization", format!("Bearer {}", api_token))
        .header(USER_AGENT, user_agent) // gh api requires a user-agent header
        .header("X-GitHub-Api-Version", "2022-11-28")
        .json(&body)
        .send()
        .await?;

    // Vérifier le status de la réponse
    if response.status().is_success() {
        println!("Success : {:?}", response.status());
        return Ok(true);
    }

    if response.status().is_client_error() {
        let status = response.status();
        println!("Erreur détaillée : {}", response.text().await?);
        return Err(format!("Erreur client: {}", status).into());
    }

    let status = response.status();
    println!("Erreur détaillée : {}", response.text().await?);
    Err(format!("Erreur serveur: {}", status).into())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let todois_api_token = env::var("TODOIST_API_TOKEN")
        .expect("La variable d'environnement TODOIST_API_TOKEN n'est pas définie");
    let github_api_token = env::var("GITHUB_API_TOKEN")
        .expect("La variable d'environnement GITHUB_API_TOKEN n'est pas définie");
    let github_user_agent = env::var("GITHUB_USER_AGENT")
        .expect("La variable d'environnement GITHUB_API_TOKEN n'est pas définie");
    let executor = env::var("EXECUTOR")
        .expect("La variable d'environnement EXECUTOR n'est pas définie");
    let project_id = "2332182173";

    let last_articles_blog = get_last_articles_blog(&github_api_token, &github_user_agent).await?;

    let tasks = get_todoist_tasks(&todois_api_token, project_id).await?;

    let filtered_articles: Vec<Task> = exclude_put_aside_category_tasks(tasks);

    let grouped_tasks = group_by_category(filtered_articles);

    let head_of_article = create_head_of_article(last_articles_blog);
    let body_of_article = create_body_of_article(grouped_tasks);

    let commit_message = format!("[EnVrac] - Publish Auto (envrac-rust - {}) {}", executor, Local::now().format("%Y-%m-%d-envrac.md"));
    push_new_article_blog(&github_api_token, &github_user_agent, &format!("{}\n{}", head_of_article, body_of_article), &*commit_message).await?;

    Ok(())
}

fn create_body_of_article(grouped_tasks: HashMap<Category, Vec<Task>>) -> String {
    let mut body = String::with_capacity(1024); // Pré-alloue de l'espace pour optimiser

    for (category, articles) in grouped_tasks.into_iter() {
        // Ajout d'un saut de ligne et de la catégorie
        body.push_str("\n\n## ");
        body.push_str(&category.to_string());
        body.push_str("\n");

        // Traitement de chaque article
        for article in articles {
            body.push_str("- ");
            body.push_str(&article.content);

            if !article.description.is_empty() {
                body.push_str(" - ");
                body.push_str(&article.description);
                body.push_str("\n");
            } else {
                body.push_str("\n");
            }
        }
    }

    body
}

fn create_head_of_article(last_articles_blog: Vec<Content>) -> String {
    let now = Local::now();

    let day_letter = format_day_letter(now);
    let last_article1date = format_day_letter(convert_to_datetime(last_articles_blog[0].date.clone(), now));
    let last_article2date = format_day_letter(convert_to_datetime(last_articles_blog[1].date.clone(), now));

    let content_blog = format!("---
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
- [[En Vrac] - {lastArticle1date}](https://blog.victorprouff.fr/en-vracs/{lastArticle1name}/)
- [[En Vrac] - {lastArticle2date}](https://blog.victorprouff.fr/en-vracs/{lastArticle2name}/)",
                                   dayLetter = day_letter,
                                   year = now.year(),
                                   month = now.format("%m").to_string(),
                                   day = now.format("%d").to_string(),
                                   lastArticle1date = last_article1date,
                                   lastArticle1name = last_articles_blog[0].name,
                                   lastArticle2date = last_article2date,
                                   lastArticle2name = last_articles_blog[1].name);
    content_blog
}

fn convert_to_datetime(last_articles_blog: String, now: DateTime<Local>) -> DateTime<Local> {
    let naive_date = NaiveDate::parse_from_str(&*last_articles_blog.clone(), "%Y-%m-%d").unwrap();
    let datetime = naive_date.and_hms_opt(0, 0, 0).unwrap();
    let datetime_utc = DateTime::<Local>::from_naive_utc_and_offset(datetime, now.offset().clone());
    datetime_utc
}

fn format_day_letter(now: DateTime<Local>) -> String {
    format!("{} {}",
            now.format("%d").to_string(),
            MOIS[now.month0() as usize]).to_string()
}

fn group_by_category(filtered_articles: Vec<Task>) -> HashMap<Category, Vec<Task>> {
    let mut grouped_articles: HashMap<Category, Vec<Task>> = HashMap::new();

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

fn exclude_put_aside_category_tasks(articles: Vec<Task>) -> Vec<Task> {
    articles
        .into_iter()
        .filter(|article| !matches!(article.category, Some(Category::PutAside)))
        .collect()
}
