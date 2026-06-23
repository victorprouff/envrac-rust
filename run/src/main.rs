use run::Task;
use run::models::{Author, Category, Committer, GithubRequest};
use std::collections::HashMap;
use std::env;
use base64::Engine;
use base64::engine::general_purpose;
use chrono::{Local, Datelike, DateTime, NaiveDate};
use clap::{Parser, Subcommand};
use reqwest::header::USER_AGENT;
use serde::Deserialize;

const MOIS: [&str; 12] = ["Janvier", "Février", "Mars", "Avril", "Mai", "Juin",
    "Juillet", "Août", "Septembre", "Octobre", "Novembre", "Décembre"];

async fn get_todoist_tasks(
    api_token: &str,
    project_id: &str,
) -> Result<Vec<Task>, Box<dyn std::error::Error>> {
    let url = format!(
        "https://api.todoist.com/api/v1/tasks?project_id={}",
        project_id
    );

    let client = reqwest::Client::new();
    let max_retries = 3;
    let mut last_error = String::new();

    for attempt in 0..=max_retries {
        if attempt > 0 {
            let delay = std::time::Duration::from_secs(2u64.pow(attempt as u32));
            println!("TODOIST - Tentative {}/{} après {}s d'attente...", attempt + 1, max_retries + 1, delay.as_secs());
            tokio::time::sleep(delay).await;
        }

        let response = match client
            .get(&url)
            .header("Authorization", format!("Bearer {}", api_token))
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                last_error = format!("TODOIST - Erreur réseau: {}", e);
                println!("{}", last_error);
                continue;
            }
        };

        if response.status().is_success() {
            #[derive(Deserialize)]
            struct TodoistResponse { results: Vec<Task> }
            let body: TodoistResponse = response.json().await?;
            let mut articles = body.results;
            for article in &mut articles {
                article.post_deserialize();
            }
            return Ok(articles);
        }

        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        last_error = format!("TODOIST - Erreur {}: {}", status, body);
        println!("{}", last_error);

        if !status.is_server_error() {
            return Err(last_error.into());
        }
    }

    Err(last_error.into())
}

#[derive(Deserialize, Debug)]
struct Content {
    name: String,
    r#type: String,
    #[serde(skip_deserializing)]
    #[serde(default)]
    date: String,
}

async fn get_last_articles_blog(api_token: &str, user_agent: &str) -> Result<Vec<Content>, Box<dyn std::error::Error>> {
    let base_url = "https://api.github.com/repos/victorprouff/blog-hugo/contents/content/en-vracs";
    let client = reqwest::Client::new();

    let response = client
        .get(base_url)
        .header("Accept", "application/vnd.github+json")
        .header("Authorization", format!("Bearer {}", api_token))
        .header(USER_AGENT, user_agent)
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        println!("GITHUB (get_last_articles_blog) - Erreur {} : {}", status, body);
        return Err(format!("GITHUB (get_last_articles_blog) - Erreur {}: {}", status, body).into());
    }

    let mut year_dirs: Vec<Content> = response.json().await?;
    year_dirs.retain(|e| e.r#type == "dir");
    year_dirs.sort_by(|a, b| b.name.cmp(&a.name));

    let mut all_articles: Vec<Content> = Vec::new();

    for year_dir in year_dirs.iter().take(2) {
        let year_url = format!("{}/{}", base_url, year_dir.name);
        let response = client
            .get(&year_url)
            .header("Accept", "application/vnd.github+json")
            .header("Authorization", format!("Bearer {}", api_token))
            .header(USER_AGENT, user_agent)
            .header("X-GitHub-Api-Version", "2022-11-28")
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            println!("GITHUB (get_last_articles_blog/{}) - Erreur {} : {}", year_dir.name, status, body);
            return Err(format!("GITHUB (get_last_articles_blog/{}) - Erreur {}: {}", year_dir.name, status, body).into());
        }

        let mut articles: Vec<Content> = response.json().await?;
        articles.retain(|c| c.r#type == "file" && c.name != "_index.md");
        all_articles.extend(articles);

        if all_articles.len() >= 2 {
            break;
        }
    }

    all_articles.sort_by(|a, b| b.name.cmp(&a.name));
    all_articles.truncate(2);

    for content in &mut all_articles {
        content.name = content.name.replace(".md", "").to_lowercase();
        content.date = content.name[..10].to_string();
    }
    Ok(all_articles)
}

async fn get_file_sha(api_token: &str, user_agent: &str, file_url: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
    #[derive(Deserialize)]
    struct FileInfo { sha: String }

    let client = reqwest::Client::new();
    let response = client
        .get(file_url)
        .header("Accept", "application/vnd.github+json")
        .header("Authorization", format!("Bearer {}", api_token))
        .header(USER_AGENT, user_agent)
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await?;

    if response.status() == 404 {
        return Ok(None);
    }
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("GITHUB (get_file_sha) - Erreur {}: {}", status, body).into());
    }

    let info: FileInfo = response.json().await?;
    Ok(Some(info.sha))
}

async fn push_new_article_blog(api_token: &str, user_agent: &str, content: &str, commit_message: &str) -> Result<bool, Box<dyn std::error::Error>> {
    println!("Pushing new article to blog...");

    let now = Local::now();
    let year = now.year();
    let file_name = format!("{}.md", now.format("%Y-%m-%d-envrac"));
    let base_url = "https://api.github.com/repos/victorprouff/blog-hugo/contents/content/en-vracs";
    let file_url = format!("{}/{}/{}", base_url, year, file_name);

    let sha = get_file_sha(api_token, user_agent, &file_url).await?;

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
        branch: "main".to_string(),
        sha,
    };

    let client = reqwest::Client::new();
    let response = client
        .put(file_url)
        .header("Accept", "application/vnd.github+json")
        .header("Authorization", format!("Bearer {}", api_token))
        .header(USER_AGENT, user_agent)
        .header("X-GitHub-Api-Version", "2022-11-28")
        .json(&body)
        .send()
        .await?;

    if response.status().is_success() {
        println!("Success : {:?}", response.status());
        return Ok(true);
    }

    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    println!("GITHUB (push_new_article_blog) - Erreur {} : {}", status, body);
    Err(format!("GITHUB (push_new_article_blog) - Erreur {}: {}", status, body).into())
}

#[derive(Parser)]
#[command(name = "envrac", about = "Génère et publie les articles En Vrac")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Génère l'article et le publie sur le blog
    Publish,
    /// Génère l'article et l'affiche dans le terminal sans publier
    DryRun,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Publish => cmd_publish().await,
        Commands::DryRun => cmd_dry_run().await,
    };

    if let Err(e) = result {
        eprintln!("Erreur : {}", e);
        std::process::exit(1);
    }
}

async fn cmd_dry_run() -> Result<(), Box<dyn std::error::Error>> {
    let article = generate_article_content().await?;
    println!("{}", article);
    Ok(())
}

async fn cmd_publish() -> Result<(), Box<dyn std::error::Error>> {
    let github_api_token = env::var("GITHUB_API_TOKEN")
        .expect("La variable d'environnement GITHUB_API_TOKEN n'est pas définie");
    let github_user_agent = env::var("GITHUB_USER_AGENT")
        .expect("La variable d'environnement GITHUB_USER_AGENT n'est pas définie");
    let executor = env::var("EXECUTOR")
        .expect("La variable d'environnement EXECUTOR n'est pas définie");

    let article = generate_article_content().await?;

    let commit_message = format!("[EnVrac] - Publish Auto (envrac-rust - {}) {}", executor, Local::now().format("%Y-%m-%d-envrac.md"));
    push_new_article_blog(&github_api_token, &github_user_agent, &article, &commit_message).await?;

    Ok(())
}

async fn generate_article_content() -> Result<String, Box<dyn std::error::Error>> {
    let todoist_api_token = env::var("TODOIST_API_TOKEN")
        .expect("La variable d'environnement TODOIST_API_TOKEN n'est pas définie");
    let github_api_token = env::var("GITHUB_API_TOKEN")
        .expect("La variable d'environnement GITHUB_API_TOKEN n'est pas définie");
    let github_user_agent = env::var("GITHUB_USER_AGENT")
        .expect("La variable d'environnement GITHUB_USER_AGENT n'est pas définie");

    let project_id = "6V79vpFpwHpQpRJm";

    let last_articles_blog = get_last_articles_blog(&github_api_token, &github_user_agent).await?;
    let tasks = get_todoist_tasks(&todoist_api_token, project_id).await?;

    let filtered_articles: Vec<Task> = exclude_put_aside_category_tasks(tasks);
    let grouped_tasks = group_by_category(filtered_articles);

    let head_of_article = create_head_of_article(last_articles_blog);
    let body_of_article = create_body_of_article(grouped_tasks);

    Ok(format!("{}\n{}", head_of_article, body_of_article))
}

fn create_body_of_article(grouped_tasks: HashMap<Category, Vec<Task>>) -> String {
    let mut body = String::with_capacity(1024);

    for (category, articles) in grouped_tasks.into_iter() {
        body.push_str("\n\n## ");
        body.push_str(&category.to_string());
        body.push_str("\n");

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

    format!("---
title: \"[En Vrac] - {dayLetter}\"
description: \"En vrac du {dayLetter}. Mes découvertes, articles, vidéos et écoute qui m'ont intéressé et que je veux partager.\"
summary: \"En vrac du {dayLetter}. Mes découvertes, articles, vidéos et écoute qui m'ont intéressé et que je veux partager.\"
date: {year}-{month}-{day}T05:00:03+01:00
categories: [ \"En vrac\" ]
draft: false
---

Hello ! 😊

Comme chaque semaine, vous pouvez retrouver ici des liens d'articles de vidéos ou de podcast que j'ai découvert au fil de ma veille quotidienne et que j'aimerais partager avec vous. 😀

Les deux derniers EnVrac :
- [[En Vrac] - {lastArticle1date}](https://blog.victorprouff.fr/en-vracs/{lastArticle1year}/{lastArticle1name}/)
- [[En Vrac] - {lastArticle2date}](https://blog.victorprouff.fr/en-vracs/{lastArticle2year}/{lastArticle2name}/)",
           dayLetter = day_letter,
           year = now.year(),
           month = now.format("%m").to_string(),
           day = now.format("%d").to_string(),
           lastArticle1date = last_article1date,
           lastArticle1year = &last_articles_blog[0].name[..4],
           lastArticle1name = last_articles_blog[0].name,
           lastArticle2date = last_article2date,
           lastArticle2year = &last_articles_blog[1].name[..4],
           lastArticle2name = last_articles_blog[1].name)
}

fn convert_to_datetime(last_articles_blog: String, now: DateTime<Local>) -> DateTime<Local> {
    let naive_date = NaiveDate::parse_from_str(&*last_articles_blog.clone(), "%Y-%m-%d").unwrap();
    let datetime = naive_date.and_hms_opt(0, 0, 0).unwrap();
    DateTime::<Local>::from_naive_utc_and_offset(datetime, now.offset().clone())
}

fn format_day_letter(now: DateTime<Local>) -> String {
    format!("{} {}", now.format("%d"), MOIS[now.month0() as usize])
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
