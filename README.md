# ffizer <!-- omit in toc -->

<!-- copy badges from:
- [`repostatus.org`](https://www.repostatus.org/#active)
- [`Shields.io`: Quality metadata badges for open source projects](https://shields.io/#/)
-->

[![crates license](https://img.shields.io/crates/l/ffizer.svg)](http://creativecommons.org/publicdomain/zero/1.0/)
[![crate version](https://img.shields.io/crates/v/ffizer.svg)](https://crates.io/crates/ffizer)

[![Project Status: Active – The project has reached a stable, usable state and is being actively developed.](https://www.repostatus.org/badges/latest/active.svg)](https://www.repostatus.org/#active)
[![Actions Status](https://github.com/ffizer/ffizer/workflows/ci-flow/badge.svg)](https://github.com/ffizer/ffizer/actions)
[![test coverage](https://codecov.io/gh/ffizer/ffizer/branch/master/graph/badge.svg)](https://codecov.io/gh/ffizer/ffizer)

[![crates download](https://img.shields.io/crates/d/ffizer.svg)](https://crates.io/crates/ffizer)
![GitHub All Releases](https://img.shields.io/github/downloads/ffizer/ffizer/total.svg)

`ffizer` is a **f**iles and **f**olders initial**izer** / generator. It creates or updates any kind (or part) of project from template(s).

keywords: file generator, project template, project scaffolding, quick start, project bootstrap, project skeleton

[![asciicast: ffizer demo](https://raw.githubusercontent.com/ffizer/ffizer/master/docs/src/images/demo.gif)](https://asciinema.org/a/gIMUwo4H9X0EK0t6xhZ6ce6WZ)

- [Features](#features)
- [Usages](#usages)
  - [Install](#install)
    - [via homebrew](#via-homebrew)
    - [via github releases](#via-github-releases)
    - [via cargo](#via-cargo)
  - [Run](#run)
    - [Self upgrade the executable](#self-upgrade-the-executable)
    - [Apply a template (to create or update)](#apply-a-template-to-create-or-update)
  - [Authoring a template](#authoring-a-template)
- [Templates](#templates)
- [Build](#build)
- [Alternatives](#alternatives)
  - [Generic](#generic)
  - [Specialized](#specialized)

## Features

- *Create or update* files and folder from one (or several) template(s).
- A native executable (cli)
  - Install via download a standalone single file on system (no requirements like `python`, `ruby`, `nodejs`, `java`, ...).
  - Run as fast enough project generator.
  - Run with dry mode (useful to test).
  - Support self-upgrade.
- A rust library
  - Can be included into other tool
- Templates Authoring
  - Can be used for any file & folder generation (no specialization to one ecosystem).
  - Can start as simple as a folder to copy "as is".
  - Can use the [Handlebars] template syntax for file content, extended with functions:
    - To transform strings (toUpperCase, toLowerCase, Capitalize,...)
    - To retrieve content via http get (like `.gitignore` from [`gitignore.io`](https://gitignore.io), license from spdx)
    - ...
  - Can replace variables part in file and folder's name
  - Can be composed of other templates (applied as layer)
  - Can ignore file / folder under conditions
  - Can store the content at the root of the folder or under the sub-folder `template`
- Templates Hosting
  - On a local folder
  - On a hosted git repository (public / private, `github` / `bitbucket`/ `gitlab` / ...)
    - At the root of the repository
    - In a sub-folder of the repository
    - In any revision (branch, tag, commit)

[Suggestions are welcomes](https://github.com/ffizer/ffizer/issues/) ;-)

## Usages

### Install

```sh
curl https://raw.githubusercontent.com/ffizer/ffizer/master/scripts/getLatest.sh | sh
```

#### via homebrew

```sh
brew tap ffizer/ffizer
brew install ffizer-bin
ffizer upgrade
```

#### via github releases

Download the binary for your platform from [github releases](https://github.com/ffizer/ffizer/releases), then un-archive it and place it your PATH.

#### via cargo

```sh
cargo install ffizer --force --features cli
```

### Run

```txt
➜  ffizer --help

ffizer 1.6.0
https://github.com/ffizer/ffizer
ffizer is a files and folders initializer / generator.
It creates or updates any kind (or part) of project from template(s)

USAGE:
    ffizer [FLAGS] <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
    -v, --verbose    Verbose mode (-v, -vv (very verbose / level debug), -vvv) print on stderr

SUBCOMMANDS:
    apply      Apply a template into a target directory
    help       Prints this message or the help of the given subcommand(s)
    inspect    Inspect configuration, caches,... (wip)
    upgrade    Self upgrade ffizer executable
```

#### Self upgrade the executable

```sh
➜  ffizer upgrade
```

#### Apply a template (to create or update)

```sh
➜  ffizer apply --help

ffizer-apply 1.6.0
https://github.com/ffizer/ffizer
Apply a template into a target directory

USAGE:
    ffizer apply [FLAGS] [OPTIONS] --destination <dst-folder> --source <uri>

FLAGS:
    -h, --help                      Prints help information
        --offline                   in offline, only local templates or cached templates are used
        --x-always_default_value    should not ask for valiables values, always use defautl value or empty
                                    (experimental)

OPTIONS:
        --confirm <confirm>               ask for plan confirmation [default: Never]  [possible values:
                                          Auto, Always, Never]
    -d, --destination <dst-folder>        destination folder (created if doesn't exist)
        --rev <rev>                       git revision of the template [default: master]
        --source-subfolder <subfolder>    path of the folder under the source uri to use for template
        --update-mode <update-mode>       mode to update existing file [default: Ask]  [possible values:
                                          Ask, Keep, Override, UpdateAsRemote,
                                          CurrentAsLocal, ShowDiff, Merge]
    -s, --source <uri>                    uri / path of the template
```

- use a local folder as template

    ```sh
    ffizer apply --source $HOME/my_templates/tmpl0 --destination my_project
    ```

- use a remote git repository as template

    ```sh
    ffizer apply --source https://github.com/ffizer/template_sample.git --destination my_project
    ```

    output

    ```sh
    Configure variables

    project_name: my_project


    Plan to execute

      - mkdir "my_project/"
      - mkdir "my_project/dir_1"
      - copyraw "my_project/dir_1/file_1_1.txt"
      - mkdir "my_project/dir_2_my_project"
      - copyraw "my_project/dir_2_my_project/file_1_2.txt"
      - copyraw "my_project/file_1.txt"
      - copyrender "my_project/file_2.txt"
      - keep "my_project/file_2.txt"
      - copyrender "my_project/file_3.txt"
      - copyraw "my_project/file_4_my_project.txt"
      - copyrender "my_project/file_5_my_project.txt"
      - copyraw "my_project/file_6.hbs"
    ```
  
### Authoring a template

see [Template Authoring - ffizer](https://ffizer.github.io/ffizer/book/template_authoring.html) *WIP*

## Templates

- Any git repositories (in this case ffizer is like `git clone ...`)
- Parametrized (with variables) templates:
  - [`ffizer/templates_default`: the default collections of templates for ffizer](https://github.com/ffizer/templates_default) (WIP)
  - [`davidB31 / cg-starter-multi-rust` · GitLab](https://gitlab.com/davidB31/cg-starter-multi-rust) Project template for Multi-Bot in Rust on CodinGame.
  - [`davidB/templates`: repository to host the my collections of templates to used with ffizer.](https://github.com/davidB/templates)
  - github repo tagged [`ffizer-template`](https://github.com/topics/ffizer-template)
  - samples (used for test, demo)
  templates_default)
    - [test_1](tests/test_1/template)
    - [test_2](tests/test_2/template) (demo of usage of gitignore.io)
    - [`ffizer/template_sample`: a simple template for ffizer used for demo and test](https://github.com/ffizer/template_sample)

## Build

```sh
cargo install cargo-make --force
cargo make ci-flow
```

Update [`CHANGELOG.md`](./CHANGELOG.md)

```sh
cargo make update-changelog
git add CHANGELOG.md
git commit -m ':memo: (CHANGELOG) update'
```

Release a new version by bump `patch` (or `minor`or `major`)

```sh
cargo make publish -- patch --dry-run
cargo make publish -- patch
```

## Alternatives

### Generic

- [`Cookiecutter`](https://cookiecutter.readthedocs.io/), lot of templates, require python + pip + install dependencies on system (automatic)
- [`Cookiecutter` — Similar projects](https://cookiecutter.readthedocs.io/en/latest/readme.html#similar-projects)
- [`sethyuan/fgen`](https://github.com/sethyuan/fgen): A file generator library to be used to generate project structures, file templates and/or snippets. Templates are based on mustache. require nodejs
- [project_init](https://crates.io/crates/project_init) in rust, use mustache for templating but I have some issues with it (project template creation not obvious, github only) I could contributes but I have incompatible requirements.
- [`skeleton`](https://crates.io/crates/skeleton), good idea but no template file, more like a script.
- [`porteurbars`](https://crates.io/crates/porteurbars), very similar but I discovered it too late.

### Specialized

specialized to a platform, build tool,...

- [The web's scaffolding tool for modern webapps | Yeoman](http://yeoman.io/), nodejs ecosystem
- [JHipster - Generate your Spring Boot + Angular/React applications!](https://www.jhipster.tech/) require java, dedicated to java web ecosystem, optionnated template (not generic)
- [`Giter8`](http://www.foundweekends.org/giter8/) require java + [Conscript](http://www.foundweekends.org/conscript/index.html)
- [`Typesafe activator`](https://developer.lightbend.com/start/), require java, target Scala ecosystem
- [`Maven – Archetypes`](https://maven.apache.org/guides/introduction/introduction-to-archetypes.html) require java + maven, target maven ecosystem
- [cargo-generate](https://github.com/ashleygwilliams/cargo-generate), limited capabilities, target rust/cargo ecosystem
