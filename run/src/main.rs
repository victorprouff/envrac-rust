use serde::{Deserialize, Serialize};

// Structure pour représenter une tâche Todoist
#[derive(Debug, Serialize, Deserialize)]
struct Task {
    id: String,
    content: String,
    project_id: String,
    is_completed: bool,
}

async fn get_tasks(api_token: &str, project_id: &str) -> Result<Vec<Task>, Box<dyn std::error::Error>> {

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
    let tasks: Vec<Task> = response.json().await?;
    println!("{:?}", tasks);

    Ok(tasks)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_token = "";
    let project_id = "";

    match get_tasks(api_token, project_id).await {
        Ok(tasks) => {
            for task in tasks {
                println!("Tâche: {}", task.content);
            }
        }
        Err(e) => eprintln!("Erreur: {}", e),
    }

    Ok(())
}
