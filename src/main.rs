#![allow(clippy::new_ret_no_self)]

mod github;
mod travis;

use crate::{github::GitHub, travis::TravisCI};
use failure::Error;
use log::{error, info};
use std::time::Duration;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(
    name = "travis-migrate",
    about = "Migrate from travis-ci.org to travis-ci.com"
)]
enum CLI {
    #[structopt(name = "list", about = "list repositories that can be migrated")]
    List {
        #[structopt(name = "account")]
        account: String,
    },
    #[structopt(name = "migrate-repo", about = "migrate a repository")]
    MigrateRepo {
        #[structopt(name = "slug")]
        slug: String,
    },
    #[structopt(
        name = "migrate-account",
        about = "migrate all repositories in an account"
    )]
    MigrateAccount {
        #[structopt(name = "account")]
        account: String,
        #[structopt(name = "exclude", long = "exclude", multiple = true)]
        exclude: Vec<String>,
    },
}

fn migrate_protection_contexts(contexts: &[String]) -> Vec<&str> {
    contexts
        .iter()
        .map(|ctx| match ctx.as_str() {
            "continuos-integration/travis-ci" => "Travis CI - Branch",
            "continuos-integration/travis-ci/push" => "Travis CI - Branch",
            "continuos-integration/travis-ci/pr" => "Travis CI - Pull Request",
            other => other,
        })
        .collect()
}

fn migrate(
    travis_org: &TravisCI,
    travis_com: &TravisCI,
    github: &GitHub,
    repo: &str,
) -> Result<(), Error> {
    let crons = travis_org.list_crons(repo)?;
    info!("{}: found {} cron(s) to migrate", repo, crons.len());

    info!("{}: migrating...", repo);
    travis_com.start_migration(&repo)?;
    while !travis_com.is_migrated(&repo)? {
        std::thread::sleep(Duration::from_millis(100));
    }
    info!("{}: migration complete", repo);

    if !crons.is_empty() {
        for cron in &crons {
            travis_com.create_cron(repo, cron)?;
        }
        info!("{}: restored {} cron(s)", repo, crons.len());
    }

    for branch in github.protected_branches(repo)?.into_iter() {
        let contexts = branch.protection.required_status_checks.contexts;
        let new_contexts = migrate_protection_contexts(&contexts);
        if contexts != new_contexts {
            github.set_required_status_checks(repo, &branch.name, &new_contexts)?;
            info!(
                "{}: updated required status checks for branch `{}`",
                repo, branch.name
            );
        }
    }

    Ok(())
}

fn app() -> Result<(), Error> {
    let args = CLI::from_args();

    match args {
        CLI::List { account } => {
            let travis_com = TravisCI::new("com", std::env::var("TRAVIS_TOKEN_COM").ok())?;
            let repos = travis_com.repos_to_migrate(&account)?;
            if repos.is_empty() {
                info!("no repos to migrate found");
            } else {
                info!("repos to migrate:");
                for repo in &repos {
                    info!("{}", repo.slug);
                }
            }
        }
        CLI::MigrateRepo { slug } => {
            let travis_org = TravisCI::new("org", std::env::var("TRAVIS_TOKEN_ORG").ok())?;
            let travis_com = TravisCI::new("com", std::env::var("TRAVIS_TOKEN_COM").ok())?;
            let github = GitHub::new(std::env::var("GITHUB_TOKEN")?);
            migrate(&travis_org, &travis_com, &github, &slug)?;
        }
        CLI::MigrateAccount { account, exclude } => {
            let travis_org = TravisCI::new("org", std::env::var("TRAVIS_TOKEN_ORG").ok())?;
            let travis_com = TravisCI::new("com", std::env::var("TRAVIS_TOKEN_COM").ok())?;
            let github = GitHub::new(std::env::var("GITHUB_TOKEN")?);
            let repos = travis_com.repos_to_migrate(&account)?;
            if repos.is_empty() {
                info!("no repos to migrate found");
            } else {
                info!("{} repo(s) to migrate", repos.len());
                for repo in &repos {
                    if exclude.contains(&repo.slug) {
                        info!("skipping {}", repo.slug);
                    } else {
                        migrate(&travis_org, &travis_com, &github, &repo.slug)?;
                    }
                }
            }
        }
    }
    Ok(())
}

fn main() {
    let mut logger = env_logger::Builder::new();
    logger.filter_module("travis_migrate", log::LevelFilter::Info);
    if let Ok(content) = std::env::var("RUST_LOG") {
        logger.parse(&content);
    }
    logger.init();

    if let Err(err) = app() {
        error!("{}", err);
        for cause in err.iter_causes() {
            error!("caused by: {}", cause);
        }
    }
}
