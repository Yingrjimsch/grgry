use std::sync::Arc;

use crate::{
    config::config::Profile,
    git_api::git_providers::{call_api, get_repos_paralell, GitProvider, Repo},
};

use chrono::{DateTime, Utc};
use colored::Colorize;
use reqwest::{Client, Response};
use serde::Deserialize;
use tokio::task::block_in_place;

const PER_PAGE: i16 = 100;

#[derive(Debug, Deserialize)]
pub(crate) struct GitlabRepo {
    pub ssh_url_to_repo: String,
    pub http_url_to_repo: String,
    pub path_with_namespace: String,
    pub default_branch: Option<String>,
    pub last_activity_at: Option<DateTime<Utc>>,
    pub last_repository_activity_at: Option<DateTime<Utc>>,
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

    fn default_branch(&self) -> Option<&str> {
        self.default_branch.as_deref()
    }

    fn last_activity_at(&self) -> Option<DateTime<Utc>> {
        self.last_repository_activity_at
            .or(self.last_activity_at)
    }
}

pub struct Gitlab;
impl GitProvider for Gitlab {
    fn get_repos(
        &self,
        client: Arc<Client>,
        pat: &Option<String>,
        collection_name: &str,
        user: bool,
        active_profile: Profile,
    ) -> Vec<Box<dyn Repo>> {
        block_in_place(|| {
            let future = async {
                let collection_type: &str = match user {
                    true => "users",
                    false => "groups",
                };
                let encoded_collection_name: String =
                    urlencoding::encode(collection_name).into_owned();

                let endpoint: String = format!(
                    "{}/api/v4/{}/{}/projects",
                    active_profile.baseaddress, collection_type, encoded_collection_name
                );
                let headers: Option<Vec<(String, String)>> = match pat {
                    Some(token) => Some(vec![
                        ("Private-Token".to_string(), token.clone()),
                        ("User-Agent".to_string(), "grgry".to_string()),
                    ]),
                    None => None,
                };
                let pages: i32 =
                    self.get_page_number(Arc::clone(&client), &endpoint, headers.clone());
                let parameters: Option<Vec<(String, String)>> = Some(vec![
                    ("include_subgroups".to_string(), "true".to_string()),
                    ("simple".to_string(), "true".to_string()),
                    ("per_page".to_string(), PER_PAGE.to_string()),
                ]);
                get_repos_paralell(
                    client,
                    pages,
                    &endpoint,
                    parameters,
                    headers,
                    &active_profile.provider,
                )
                .await
            };

            // Block on the async task, so it runs to completion and returns the result.
            let repos: Vec<Box<dyn Repo>> = tokio::runtime::Handle::current().block_on(future);
            repos
        })
    }

    fn get_page_number(
        &self,
        client: Arc<Client>,
        endpoint: &str,
        headers: Option<Vec<(String, String)>>,
    ) -> i32 {
        block_in_place(|| {
            let future = async {
                let parameters: Option<Vec<(String, String)>> = Some(vec![
                    ("include_subgroups".to_string(), "true".to_string()),
                    ("simple".to_string(), "true".to_string()),
                    ("page".to_string(), "1".to_string()),
                    ("per_page".to_string(), PER_PAGE.to_string()),
                ]);
                let resp_total_repos: Response =
                    call_api(&client, endpoint, parameters.as_deref(), headers.as_deref()).await;

                if !resp_total_repos.status().is_success() {
                    let status = resp_total_repos.status();
                    let body = resp_total_repos.text().await.unwrap_or_default();
                    eprintln!(
                        "{} {} {}\nEndpoint: {}\nResponse body:\n{}",
                        "GitLab API request failed with status".red(),
                        status.to_string().red(),
                        "- cannot determine pagination.".red(),
                        endpoint,
                        body
                    );
                    return 0;
                }

                resp_total_repos
                    .headers()
                    .get("x-total-pages")
                    .and_then(|hv: &reqwest::header::HeaderValue| hv.to_str().ok())
                    .and_then(|s: &str| s.parse::<i32>().ok())
                    .unwrap_or(1)
            };
            // Block on the async task, so it runs to completion and returns the result.
            let pages: i32 = tokio::runtime::Handle::current().block_on(future);
            if pages <= 0 {
                1
            } else {
                pages
            }
        })
    }
}
