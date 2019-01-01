# travis-ci.org to travis-ci.com migrator

This repository contains a tool that can be used to migrate a repository or an
entire organization from travis-ci.org to travis-ci.com. Note that **your
Travis CI account needs to have the migration feature enabled in order to use
this tool**. Please ask Travis CI support for that. The tool tries to migrate
as much data and settings as possible. In addition to [the data migrated by
Travis itself][data-migrated], it will also migrate:

* All the cron jobs configured in the repository

You need Rust 1.31.0 or greater in order to use this tool. Made by [Pietro
Albini](https://www.pietroalbini.org) and released under the MIT license.

[data-migrated]: https://docs.travis-ci.com/user/open-source-repository-migration/#what-information-will-be-transferred-to-travis-cicom

## Travis authentication tokens

The tool needs your API keys for both `travis-ci.org` and `travis-ci.com`. If
you have the [Travis CLI](https://github.com/travis-ci/travis.rb) installed and
you're logged into it the tool will fetch the tokens automatically. Otherwise
you can provide them with the environment variables:

* `TRAVIS_TOKEN_ORG` for `travis-ci.org`
* `TRAVIS_TOKEN_COM` for `travis-ci.com`

## Usage

**REMEMBER IT'S NOT POSSIBLE TO MIGRATE A REPOSITORY BACK TO TRAVIS-CI.ORG!!!**

You can list all the repositories that can be migrated in an
account/organization with:

```
$ cargo run list rust-lang
```

You can migrate a single repository with:

```
$ cargo run migrate-repo rust-lang/rust
```

You can migrate all the repositories inan account/organization with:

```
$ cargo run migrate-account rust-lang
```
