use crate::cli::{RepoArgs, RepoCommands};
use crate::config::Config;
use crate::error::ApiError;
use serde::Deserialize;

const BASE_CATALOG_URI: &str = "/v2/_catalog";

pub async fn handler(config: &Config, args: &RepoArgs) -> Result<(), ApiError> {
    log::trace!("handler()");

    match args.command {
        RepoCommands::List => handle_list(config, args).await?,
    }

    Ok(())
}


#[derive(Deserialize)]
struct CatalogResponse {
    repositories: Vec<String>,
}

async fn handle_list(config: &Config, _args: &RepoArgs) -> Result<(), ApiError> {
    log::trace!("handle_list()");

    let mut url = config.registry_url.join(BASE_CATALOG_URI)?;
    let mut responses = Vec::new();

    loop {
        log::debug!("Using url {url}");
        let response = reqwest::get(url.clone()).await?;
        let headers = response.headers().to_owned();
        let body: CatalogResponse = response.json().await?;

        responses.push(body);

        if let Some(link_value) = headers.get(http::header::LINK) {
            let link_str = link_value.to_str()?;
            let parts:Vec<&str> = link_str.split(';').collect();
            if let Some(url_part) = parts.first() {
                if let Some(uri) = url_part.trim().strip_prefix('<').and_then(|s| s.strip_suffix('>')) {
                    url = config.registry_url.join(uri)?;
                }
            }
        } else {
            break;
        }
    }

    let repo_list: Vec<&str> = responses.iter().flat_map(|r| r.repositories.iter().map(String::as_str)).collect();
    
    for repo in repo_list {
        println!("{repo}");
    }

    Ok(())

}