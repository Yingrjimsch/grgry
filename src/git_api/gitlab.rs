use crate::{
    git_api::git_providers::{call_api, get_repos_paralell, GitProvider, Repo},
    config::config::Profile,
};

use reqwest::Response;
use serde::Deserialize;
use tokio::task::block_in_place;
const PER_PAGE: i16 = 100;

#[derive(Debug, Deserialize)]
pub(crate) struct GitlabRepo {
    pub ssh_url_to_repo: String,
    pub http_url_to_repo: String,
    pub path_with_namespace: String,
}

impl Repo for GitlabRepo {
    fn ssh_url(&self) -> &str {
        &self.ssh_url_to_repo
    }

    fn http_url(&self) -> &str {
        &self.http_url_to_repo
    }

    fn full_path(&self) -> &str {
        &self.path_with_namespace
    }
}

pub struct Gitlab;
impl GitProvider for Gitlab {
    fn get_repos(
        &self, pat: &Option<String>, collection_name: &str, user: bool, active_profile: Profile,
    ) -> Vec<Box<dyn Repo>> {
        block_in_place(|| {
            let future = async {
                let collection_type: &str = match user {
                    true => "users",
                    false => "groups",
                };
                let endpoint: String = format!(
                    "{}/api/v4/{}/{}/projects",
                    active_profile.baseaddress,
                    collection_type,
                    collection_name.replace("/", "%2F")
                );
                let headers: Option<Vec<(String, String)>> = match pat {
                    Some(token) => Some(vec![
                        ("Private-Token".to_string(), token.clone()),
                        ("User-Agent".to_string(), "grgry".to_string()),
                    ]),
                    None => None,
                };
                let pages: i32 = self.get_page_number(&endpoint, headers.clone());
                let parameters: Option<Vec<(String, String)>> = Some(vec![
                    ("include_subgroups".to_string(), "true".to_string()),
                    ("simple".to_string(), "true".to_string()),
                    ("per_page".to_string(), PER_PAGE.to_string()),
                ]);
                get_repos_paralell(pages, &endpoint, parameters, headers, &active_profile.provider).await
            };

            // Block on the async task, so it runs to completion and returns the result.
            let repos: Vec<Box<dyn Repo>> = tokio::runtime::Handle::current().block_on(future);
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
                let resp_total_repos: Response = call_api(endpoint, parameters.as_deref(), headers.as_deref()).await;
                return resp_total_repos
                    .headers()
                    .get("x-total-pages")
                    .and_then(|hv: &reqwest::header::HeaderValue| hv.to_str().ok())
                    .and_then(|s: &str| s.parse::<i32>().ok())
                    .unwrap();
            };
            // Block on the async task, so it runs to completion and returns the result.
            let repos: i32 = tokio::runtime::Handle::current().block_on(future);
            repos
        })
    }
}
