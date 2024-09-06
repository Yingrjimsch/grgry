use std::process;

use crate::{git_api::github::GithubRepo, git_api::gitlab::GitlabRepo, config::config::Profile};
use colored::Colorize;
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Client, Response,
};
use tokio::task;

use super::{github::Github, gitlab::Gitlab};

// Define the trait in a common file (e.g., `git_provider.rs`)
pub trait GitProvider {
    fn get_page_number(&self, endpoint: &str, headers: Option<Vec<(String, String)>>) -> i32;
    fn get_repos(
        &self, pat: &Option<String>, collection_name: &str, user: bool, active_profile: Profile,
    ) -> Vec<Box<dyn Repo>>;
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
    provider: &str, // Enum to distinguish between Github and Gitlab
) -> Vec<Box<dyn Repo>> {
    let mut tasks: Vec<task::JoinHandle<Result<Vec<Box<dyn Repo>>, reqwest::Error>>> = Vec::new();

    for page in 1..=pages {
        let endpoint_clone: String = endpoint.to_string();
        let mut parameters_clone: Option<Vec<(String, String)>> = parameters.clone();
        let headers_clone: Option<Vec<(String, String)>> = headers.clone();
        let provider: String = provider.to_string();
        if let Some(params) = &mut parameters_clone {
            params.push(("page".to_string(), page.to_string()));
        }

        tasks.push(task::spawn(async move {
            let response: Response =
                call_api(&endpoint_clone, parameters_clone.as_deref(), headers_clone.as_deref()).await;
            let repos: Vec<Box<dyn Repo>> = match provider.as_str() {
                "gitlab" => response
                    .json::<Vec<GitlabRepo>>()
                    .await?
                    .into_iter()
                    .map(|repo: GitlabRepo| Box::new(repo) as Box<dyn Repo>)
                    .collect(),
                "github" => response
                    .json::<Vec<GithubRepo>>()
                    .await?
                    .into_iter()
                    .map(|repo: GithubRepo| Box::new(repo) as Box<dyn Repo>)
                    .collect(),
                _ => unreachable!(),
            };
            Ok::<Vec<Box<dyn Repo>>, reqwest::Error>(repos)
        }));
    }

    let mut all_repos: Vec<Box<dyn Repo>> = Vec::new();
    for task in tasks {
        let repos: Vec<Box<dyn Repo>> = task.await.unwrap().unwrap();
        all_repos.extend(repos);
    }

    all_repos
}

pub async fn call_api(
    endpoint: &str, parameters: Option<&[(String, String)]>, headers: Option<&[(String, String)]>,
) -> Response {
    let client: Client = Client::builder().build().expect("error while build client");
    let mut request: reqwest::RequestBuilder = client.get(endpoint);
    // .header("User-Agent", "grgry-cli"); //TODO: check if user agent makes sense
    if let Some(params) = parameters {
        request = request.query(params)
    }

    if let Some(header_pairs) = headers {
        let mut header_map: HeaderMap = HeaderMap::new();
        for (key, value) in header_pairs {
            let header_name: HeaderName = HeaderName::from_bytes(key.as_bytes()).unwrap();
            let header_value: HeaderValue = HeaderValue::from_str(value).unwrap();
            header_map.insert(header_name, header_value);
        }
        request = request.headers(header_map);
    }

    let response: Response = match request.send().await {
        Ok(resp) => resp,
        Err(err) => {
            eprintln!("{} {:?}", "Error sending request:".red(), err);
            process::exit(1);
        }
    };

    return response;
}

pub fn get_provider(provider_type: &str) -> Box<dyn GitProvider> {
    return match provider_type {
        "gitlab" => Box::new(Gitlab),
        "github" => Box::new(Github),
        _ => {
            println!("{} {} {}", "The provider type".red(), provider_type.red(), "is not supported or does not exist, for further https://github.com/Yingrjimsch/grgry/issues/new?assignees=&labels=question&projects=&template=FEATURE-REQUEST.yml");
            unreachable!()
        },
    }
}
