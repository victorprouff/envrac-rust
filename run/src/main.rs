use run::Task;
use run::models::Category;
use std::collections::HashMap;
use std::env;
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let todois_api_token = env::var("TODOIST_API_TOKEN")
        .expect("La variable d'environnement TODOIST_API_TOKEN n'est pas définie");
    let github_api_token = env::var("GITHUB_API_TOKEN")
        .expect("La variable d'environnement GITHUB_API_TOKEN n'est pas définie");
    let github_user_agent = env::var("GITHUB_USER_AGENT")
        .expect("La variable d'environnement GITHUB_API_TOKEN n'est pas définie");
    let project_id = "2332182173";

    let last_articles_blog = get_last_articles_blog(&github_api_token, &github_user_agent).await?;

    let tasks = get_todoist_tasks(&todois_api_token, project_id).await?;

    let filtered_articles: Vec<Task> = exclude_put_aside_category_tasks(tasks);

    let grouped_tasks = group_by_category(filtered_articles);

    let head_of_article = create_head_of_article(last_articles_blog);
    let body_of_article = create_body_of_article(grouped_tasks);

    println!("{} \n {}", head_of_article, body_of_article);



    Ok(())
}

fn create_body_of_article(grouped_tasks: HashMap<Category, Vec<Task>>) -> String {
    let mut body = String::new();

    // Afficher les articles groupés par catégorie
    for (category, articles) in grouped_tasks.iter() {
        body.push_str(&format!("\n\n## {}\n", category.to_string()));
        for article in articles {
            body.push_str(&format!("- {}", article.content));
            if !article.description.is_empty() {
                body.push_str(&format!(" - {}\n", article.description));
            } else {
                body.push_str(&"\n".to_string());
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
