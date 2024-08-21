use crate::git_providers::GitProvider;
use std::{convert, process};

use reqwest::{header::{HeaderMap, HeaderName, HeaderValue}, Client, Response};
use serde::Deserialize;
use tokio::{spawn, task::{self, block_in_place}};
const PER_PAGE: i16 = 100;

pub struct GitLab;

impl GitProvider for GitLab {
    fn get_repos(&self, pat: &Option<String>, base_address: &str, collection_name: &str) -> Vec<GitlabRepo> {
        block_in_place(|| {
            let future = async {
                let endpoint: String = format!("{}/api/v4/groups/{}/projects", base_address, collection_name.replace("/", "%2F"));
                let headers: Option<Vec<(String, String)>> = match pat {
                    Some(token) => Some(vec![("Private-Token".to_string(), token.clone())]),
                    None => None,
                };
                let pages: i32 = self.get_page_number(&endpoint, headers.clone());
                let parameters: Option<Vec<(String, String)>> = Some(vec![
                    ("include_subgroups".to_string(), "true".to_string()),
                    ("simple".to_string(), "true".to_string()),
                    ("page".to_string(), pages.to_string()),
                    ("per_page".to_string(), PER_PAGE.to_string()),
                ]);
                get_gitlab_repos_paralell(pages, &endpoint, parameters, headers).await
            };
    
            // Block on the async task, so it runs to completion and returns the result.
            let repos = tokio::runtime::Handle::current().block_on(future);
            repos
        })
    }

    fn get_page_number(&self, endpoint: &str, headers: Option<Vec<(String, String)>>) -> i32 {
        block_in_place(|| {
            let future = async {
                let parameters: Option<Vec<(String, String)>> = Some(vec![
                    ("include_subgroups".to_string(), "true".to_string()),
                    ("simple".to_string(), "true".to_string()),
                    ("page".to_string(), "1".to_string()),
                    ("per_page".to_string(), PER_PAGE.to_string()),
                ]);
                let resp_total_repos: Response = call_gitlab_api(endpoint, parameters.as_deref(), headers.as_deref()).await;
                return resp_total_repos.headers().get("x-total-pages").and_then(|hv| hv.to_str().ok()).and_then(|s| s.parse::<i32>().ok()).unwrap();
            };
            // Block on the async task, so it runs to completion and returns the result.
            let repos = tokio::runtime::Handle::current().block_on(future);
            repos
        })
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct GitlabRepo {
    pub name: String,
    pub ssh_url_to_repo: String,
    pub http_url_to_repo: String,
    pub path: String,
    pub path_with_namespace: String
}

pub async fn get_gitlab_repos_paralell(pages: i32, endpoint: &str, parameters: Option<Vec<(String, String)>>, headers: Option<Vec<(String, String)>>) -> Vec<GitlabRepo> {
    let mut tasks: Vec<task::JoinHandle<Result<Vec<GitlabRepo>, reqwest::Error>>> = Vec::new();
    for page in 1..=pages {
        let endpoint_clone = endpoint.to_string();
        let mut parameters_clone = parameters.clone();
        let headers_clone = headers.clone();

        if let Some(params) = &mut parameters_clone {
            params.push(("page".to_string(), page.to_string()))
            
        }
        tasks.push(task::spawn(async move {
          let response: Response = call_gitlab_api(&endpoint_clone, parameters_clone.as_deref(), headers_clone.as_deref()).await;
          let repos: Vec<GitlabRepo> = response.json().await?;
            Ok::<Vec<GitlabRepo>, reqwest::Error>(repos)
        }));
    }
    // Wait for all tasks to complete
    let mut all_repos = Vec::new();
    for task in tasks {
        let repos = task.await;
        all_repos.extend(repos.unwrap().unwrap());
    }

    return all_repos
}

pub async fn call_gitlab_api(endpoint: &str, parameters: Option<&[(String, String)]>, headers: Option<&[(String, String)]>) -> Response {
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
