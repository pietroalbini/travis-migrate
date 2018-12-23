use failure::{bail, Error};
use log::{debug, info};
use reqwest::{
    header::{HeaderName, AUTHORIZATION, USER_AGENT},
    Client, Method, RequestBuilder,
};
use std::process::Command;

#[derive(serde_derive::Deserialize)]
struct PaginationLink {
    #[serde(rename = "@href")]
    href: String,
}

#[derive(serde_derive::Deserialize)]
struct Pagination {
    next: Option<PaginationLink>,
}

#[derive(serde_derive::Deserialize)]
struct Common {
    #[serde(rename = "@pagination")]
    pagination: Pagination,
}

#[derive(serde_derive::Deserialize)]
struct Repositories {
    #[serde(flatten)]
    common: Common,
    repositories: Vec<Repository>,
}

#[derive(serde_derive::Deserialize)]
pub(crate) struct Repository {
    pub(crate) slug: String,
    migration_status: Option<String>,
}

#[derive(serde_derive::Deserialize)]
struct Crons {
    #[serde(flatten)]
    common: Common,
    crons: Vec<Cron>,
}

#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CronInterval {
    Daily,
    Weekly,
    Monthly,
}

#[derive(serde_derive::Deserialize)]
pub(crate) struct Branch {
    name: String,
}

#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub(crate) struct Cron {
    #[serde(skip_serializing)]
    branch: Branch,
    interval: CronInterval,
    dont_run_if_recent_build_exists: bool,
}

pub(crate) struct TravisCI {
    tld: &'static str,
    token: String,
    client: Client,
}

impl TravisCI {
    pub(crate) fn new(tld: &'static str, token: Option<String>) -> Result<Self, Error> {
        let token = if let Some(token) = token {
            token
        } else {
            info!("fetching API token for travis-ci.{}", tld);
            let output = Command::new("travis")
                .arg("token")
                .arg(format!("--{}", tld))
                .arg("--no-interactive")
                .output()?;
            if !output.status.success() {
                bail!(
                    "failed to get the travis-ci.{} token: {}",
                    tld,
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            String::from_utf8(output.stdout)?.trim().to_string()
        };
        Ok(TravisCI {
            tld,
            token,
            client: Client::new(),
        })
    }

    fn build_request(&self, method: Method, url: &str) -> RequestBuilder {
        let tmp_url;
        let mut url = url.trim_start_matches('/');
        if !url.starts_with("https://") {
            tmp_url = format!("https://api.travis-ci.{}/{}", self.tld, url);
            url = &tmp_url;
        }
        debug!("{} {}", method, url);
        self.client
            .request(method, url)
            .header(USER_AGENT, "pietroalbini/travis-migrate")
            .header(AUTHORIZATION, format!("token {}", self.token))
            .header(HeaderName::from_static("travis-api-version"), "3")
    }

    fn paginated<F>(&self, method: &Method, url: &str, mut f: F) -> Result<(), Error>
    where
        F: FnMut(RequestBuilder) -> Result<Common, Error>,
    {
        let mut common = f(self.build_request(method.clone(), url))?;
        while let Some(link) = common.pagination.next {
            common = f(self.build_request(method.clone(), &link.href))?;
        }
        Ok(())
    }

    fn repo_name(&self, name: &str) -> String {
        name.replace("/", "%2F")
    }

    pub(crate) fn repos_to_migrate(&self, login: &str) -> Result<Vec<Repository>, Error> {
        let mut repos = Vec::new();
        self.paginated(&Method::GET, &format!("owner/{}/repos", login), |req| {
            let mut resp: Repositories = req
                .form(&[("active_on_org", "true")])
                .send()?
                .error_for_status()?
                .json()?;
            repos.append(&mut resp.repositories);
            Ok(resp.common)
        })?;
        Ok(repos)
    }

    pub(crate) fn start_migration(&self, repo: &str) -> Result<(), Error> {
        let _ = self
            .build_request(
                Method::POST,
                &format!("repo/{}/migrate", self.repo_name(repo)),
            )
            .send()?
            .error_for_status()?;
        Ok(())
    }

    pub(crate) fn is_migrated(&self, repo: &str) -> Result<bool, Error> {
        let repo: Repository = self
            .build_request(Method::GET, &format!("repo/{}", self.repo_name(repo)))
            .send()?
            .error_for_status()?
            .json()?;
        Ok(repo.migration_status.as_ref().map(|s| s.as_str()) == Some("migrated"))
    }

    pub(crate) fn list_crons(&self, repo: &str) -> Result<Vec<Cron>, Error> {
        let mut crons = Vec::new();
        self.paginated(
            &Method::GET,
            &format!("repo/{}/crons", self.repo_name(repo)),
            |req| {
                let mut resp: Crons = req.send()?.error_for_status()?.json()?;
                crons.append(&mut resp.crons);
                Ok(resp.common)
            },
        )?;
        Ok(crons)
    }

    pub(crate) fn create_cron(&self, repo: &str, cron: &Cron) -> Result<(), Error> {
        let _ = self
            .build_request(
                Method::POST,
                &format!(
                    "repo/{}/branch/{}/cron",
                    self.repo_name(repo),
                    cron.branch.name
                ),
            )
            .json(cron)
            .send()?
            .error_for_status()?;
        Ok(())
    }
}
