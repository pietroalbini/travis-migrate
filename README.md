# travis-ci.org to travis-ci.com migrator

<p align="center"><b>:warning::warning: &nbsp;
It's not possible to migrate repositories back to travis-ci.org
&nbsp; :warning::warning:</b></p>

`travis-migrate` is a tool that automatically migrates repositories or whole
accounts/organizations from [travis-ci.org][org] to [travis-ci.com][com], while
trying to preserve as much data and settings as possible. It was built by the
Rust Infrastructure team to migrate all the repositories in our organizations.

In addition to the [data migrated by Travis itself][data-migrated], the tool:

* Migrates all the cron jobs configured in the repository
* Migrates the required status checks in the repository's protected branches

> When the tool was written the Migration API was limited to beta testers. If
> you can't access it you need to contact Travis Support and ask them to enable
> it on the users/organizations you want to migrate.

You need Rust 1.31.0 or greater in order to use this tool. Made by [Pietro
Albini](https://www.pietroalbini.org) and released under the MIT license.

[data-migrated]: https://docs.travis-ci.com/user/open-source-repository-migration/#what-information-will-be-transferred-to-travis-cicom
[org]: https://travis-ci.org
[com]: https://travis-ci.com

## API authentication keys

The tool needs the following API keys:

* `GITHUB_TOKEN`: a personal access token of a GitHub account that has **full
  admin access** to all the repositories
* `TRAVIS_TOKEN_ORG`: the `travis-ci.org` API key of the account you want to
  use to perform the migration
* `TRAVIS_TOKEN_COM`: the `travis-ci.com` API key of the account you want to
  use to perform the migration

If you have the [Travis CLI][travis-cli] installed you can omit the Travis
environment variables, since the tool will call the CLI to fetch the tokens
automatically.

[travis-cli]: https://github.com/travis-ci/travis.rb

## Usage

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
