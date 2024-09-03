use regex::Regex;
use reqwest::Response;
use serde::Deserialize;
use tokio::task::block_in_place;

use crate::{
    profile::config::Profile,
    git_api::git_providers::{call_api, get_repos_paralell, GitProvider, Repo},
};
const PER_PAGE: i16 = 100;

#[derive(Debug, Deserialize)]
pub struct GithubRepo {
    pub ssh_url: String,
    pub clone_url: String,
    pub full_name: String,
}

impl Repo for GithubRepo {
    fn ssh_url(&self) -> &str {
        &self.ssh_url
    }

    fn http_url(&self) -> &str {
        &self.clone_url
    }

    fn full_path(&self) -> &str {
        &self.full_name
    }
}

pub struct Github;
impl GitProvider for Github {
    fn get_repos(
        &self,
        pat: &Option<String>,
        collection_name: &str,
        user: bool,
        active_profile: Profile,
    ) -> Vec<Box<dyn Repo>> {
        block_in_place(|| {
            let future = async {
                let collection_searchstring: &str = match user {
                    true => {
                        if active_profile.username == collection_name && active_profile.token != ""
                        {
                            "user"
                        } else {
                            &format!("users/{}", collection_name)
                        }
                    }
                    false => &format!("orgs/{}", collection_name),
                };
                let endpoint: String = format!(
                    "{}/{}/repos",
                    &active_profile.baseaddress, collection_searchstring
                ); //here the replace / --> %2F is not done because Github projects are top level on org or on user
                let headers: Option<Vec<(String, String)>> = match pat {
                    Some(token) => Some(vec![
                        ("Authorization".to_string(), token.clone()),
                        ("User-Agent".to_string(), "grgry".to_string()),
                    ]),
                    None => None,
                };
                let pages: i32 = self.get_page_number(&endpoint, headers.clone());
                let parameters: Option<Vec<(String, String)>> =
                    Some(vec![("per_page".to_string(), PER_PAGE.to_string())]);
                get_repos_paralell(
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

    fn get_page_number(&self, endpoint: &str, headers: Option<Vec<(String, String)>>) -> i32 {
        block_in_place(|| {
            let future = async {
                let parameters: Option<Vec<(String, String)>> = Some(vec![
                    ("page".to_string(), "1".to_string()),
                    ("per_page".to_string(), PER_PAGE.to_string()),
                ]);
                let resp_total_repos: Response =
                    call_api(endpoint, parameters.as_deref(), headers.as_deref()).await;
                let pages: i32 = match resp_total_repos.headers().get("link") {
                    Some(page) => {
                        let re: Regex = Regex::new(r"page=(\d+)").unwrap();
                        re.captures_at(page.to_str().ok().unwrap(), 3)
                            .unwrap()
                            .get(1)
                            .unwrap()
                            .as_str()
                            .parse::<i32>()
                            .ok()
                            .unwrap()
                    }
                    None => 1,
                };
                return pages;
            };
            // Block on the async task, so it runs to completion and returns the result.
            let repos: i32 = tokio::runtime::Handle::current().block_on(future);
            repos
        })
    }
}
