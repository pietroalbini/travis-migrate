use failure::Error;
use hyper_old_types::header::{Link, RelationType};
use log::debug;
use reqwest::{
    header::{AUTHORIZATION, LINK, USER_AGENT},
    Client, Method, RequestBuilder, Response,
};
use serde_derive::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
pub(crate) struct Branch {
    pub(crate) name: String,
    pub(crate) protection: BranchProtection,
}

#[derive(Debug, Deserialize)]
pub(crate) struct BranchProtection {
    pub(crate) required_status_checks: RequiredStatusChecks,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RequiredStatusChecks {
    pub(crate) contexts: Vec<String>,
}

pub(crate) struct GitHub {
    token: String,
    client: Client,
}

impl GitHub {
    pub(crate) fn new(token: String) -> Self {
        GitHub {
            token,
            client: Client::new(),
        }
    }

    fn build_request(&self, method: Method, url: &str) -> RequestBuilder {
        let tmp_url;
        let mut url = url.trim_start_matches('/');
        if !url.starts_with("https://") {
            tmp_url = format!("https://api.github.com/{}", url);
            url = &tmp_url;
        }
        debug!("{} {}", method, url);
        self.client
            .request(method, url)
            .header(USER_AGENT, "pietroalbini/travis-migrate")
            .header(AUTHORIZATION, format!("token {}", self.token))
    }

    fn paginated<F>(&self, method: &Method, url: String, mut f: F) -> Result<(), Error>
    where
        F: FnMut(Response) -> Result<(), Error>,
    {
        let mut next = Some(url);
        while let Some(next_url) = next.take() {
            let resp = self
                .build_request(method.clone(), &next_url)
                .send()?
                .error_for_status()?;

            // Extract the next page
            if let Some(links) = resp.headers().get(LINK) {
                let links: Link = links.to_str()?.parse()?;
                for link in links.values() {
                    if link
                        .rel()
                        .map(|r| r.iter().any(|r| *r == RelationType::Next))
                        .unwrap_or(false)
                    {
                        next = Some(link.link().to_string());
                        break;
                    }
                }
            }

            f(resp)?;
        }
        Ok(())
    }

    pub(crate) fn protected_branches(&self, repo: &str) -> Result<Vec<Branch>, Error> {
        let url = format!("repos/{}/branches?protected=true", repo);
        let mut branches = Vec::new();
        self.paginated(&Method::GET, url, |mut resp| {
            let mut content: Vec<Branch> = resp.json()?;
            branches.append(&mut content);
            Ok(())
        })?;
        Ok(branches)
    }

    pub(crate) fn set_required_status_checks(
        &self,
        repo: &str,
        branch: &str,
        contexts: &[&str],
    ) -> Result<(), Error> {
        let url = format!(
            "repos/{}/branches/{}/protection/required_status_checks",
            repo, branch
        );
        self.build_request(Method::PATCH, &url)
            .json(&json!({
                "contexts": contexts,
            }))
            .send()?
            .error_for_status()?;
        Ok(())
    }
}
