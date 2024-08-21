use crate::gitlab::GitlabRepo;

// Define the trait in a common file (e.g., `git_provider.rs`)
pub trait GitProvider {
    fn get_page_number(&self, endpoint: &str, headers: Option<Vec<(String, String)>>) -> i32;
    fn get_repos(&self, pat: &Option<String>, base_address: &str, collection_name: &str) -> Vec<GitlabRepo>;
}
