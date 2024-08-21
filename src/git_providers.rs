use std::process;

use reqwest::{header::{HeaderMap, HeaderName, HeaderValue}, Client, Response};
use tokio::task;

use crate::{github::GithubRepo, gitlab::GitlabRepo};

// Define the trait in a common file (e.g., `git_provider.rs`)
pub trait GitProvider {
    fn get_page_number(&self, endpoint: &str, headers: Option<Vec<(String, String)>>) -> i32;
    fn get_repos(&self, pat: &Option<String>, base_address: &str, collection_name: &str, provider: &str) -> Vec<Box<dyn Repo>>;
}

pub trait Repo: Send + Sync {
    fn ssh_url(&self) -> &str;
    fn http_url(&self) -> &str;
    fn full_path(&self) -> &str;
}


pub async fn get_repos_paralell(
    pages: i32,
    endpoint: &str,
    parameters: Option<Vec<(String, String)>>,
    headers: Option<Vec<(String, String)>>,
    manager_type: &str,  // Enum to distinguish between GitLab and GitHub
) -> Vec<Box<dyn Repo>> {
    let mut tasks: Vec<task::JoinHandle<Result<Vec<Box<dyn Repo>>, reqwest::Error>>> = Vec::new();
    
    for page in 1..=pages {
        let endpoint_clone = endpoint.to_string();
        let mut parameters_clone = parameters.clone();
        let headers_clone = headers.clone();
        let provider: String = manager_type.to_string();
        if let Some(params) = &mut parameters_clone {
            params.push(("page".to_string(), page.to_string()));
        }

        tasks.push(task::spawn(async move {
            let response: Response = call_api(&endpoint_clone, parameters_clone.as_deref(), headers_clone.as_deref()).await;
            println!("{:?}", response);
            let repos: Vec<Box<dyn Repo>> = match provider.as_str() {
                "gitlab" => response.json::<Vec<GitlabRepo>>().await?
                    .into_iter()
                    .map(|repo| Box::new(repo) as Box<dyn Repo>)
                    .collect(),
                "github" => response.json::<Vec<GithubRepo>>().await?
                    .into_iter()
                    .map(|repo| Box::new(repo) as Box<dyn Repo>)
                    .collect(),
                _ => unreachable!()
            };
            Ok::<Vec<Box<dyn Repo>>, reqwest::Error>(repos)
        }));
    }

    let mut all_repos = Vec::new();
    for task in tasks {
        let repos = task.await.unwrap().unwrap();
        all_repos.extend(repos);
    }

    all_repos
}

pub async fn call_api(endpoint: &str, parameters: Option<&[(String, String)]>, headers: Option<&[(String, String)]>) -> Response {
    let client = Client::builder().build().expect("error while build client");
    let mut request = client.get(endpoint);
        // .header("User-Agent", "grgry-cli"); //TODO: check if user agent makes sense
    if let Some(params) = parameters {
        request = request.query(params)
    }

    if let Some(header_pairs) = headers {
        let mut header_map = HeaderMap::new();
        for (key, value) in header_pairs {
            let header_name = HeaderName::from_bytes(key.as_bytes()).unwrap();
            let header_value = HeaderValue::from_str(value).unwrap();
            header_map.insert(header_name, header_value);
        }
        request = request.headers(header_map);
    }

    let response = match request.send().await {
        Ok(resp) => resp,
        Err(err) => {
            eprintln!("Error sending request: {:?}", err);
            process::exit(1);
        }
    };

    return response;
}