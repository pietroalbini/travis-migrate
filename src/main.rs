#![allow(clippy::new_ret_no_self)]

mod travis;

use crate::travis::TravisCI;
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
    },
}

fn migrate(travis_org: &TravisCI, travis_com: &TravisCI, repo: &str) -> Result<(), Error> {
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
            migrate(&travis_org, &travis_com, &slug)?;
        }
        CLI::MigrateAccount { account } => {
            let travis_org = TravisCI::new("org", std::env::var("TRAVIS_TOKEN_ORG").ok())?;
            let travis_com = TravisCI::new("com", std::env::var("TRAVIS_TOKEN_COM").ok())?;
            let repos = travis_com.repos_to_migrate(&account)?;
            if repos.is_empty() {
                info!("no repos to migrate found");
            } else {
                info!("{} repo(s) to migrate", repos.len());
                for repo in &repos {
                    migrate(&travis_org, &travis_com, &repo.slug)?;
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
