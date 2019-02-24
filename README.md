# ffizer

<!-- copy badges from:
- [repostatus.org](https://www.repostatus.org/#active)
- [Shields.io: Quality metadata badges for open source projects](https://shields.io/#/)
-->

[![Crates.io](https://img.shields.io/crates/l/ffizer.svg)](http://creativecommons.org/publicdomain/zero/1.0/)
[![Crates.io](https://img.shields.io/crates/v/ffizer.svg)](https://crates.io/crates/ffizer)

[![Project Status: WIP – Initial development is in progress, but there has not yet been a stable, usable release suitable for the public.](https://www.repostatus.org/badges/latest/wip.svg)](https://www.repostatus.org/#wip)
[![Build Status](https://travis-ci.com/ffizer/ffizer.svg?branch=master)](https://travis-ci.com/ffizer/ffizer)
[![codecov](https://codecov.io/gh/ffizer/ffizer/branch/master/graph/badge.svg)](https://codecov.io/gh/ffizer/ffizer)


[![Crates.io](https://img.shields.io/crates/d/ffizer.svg)](https://crates.io/crates/ffizer)
![GitHub All Releases](https://img.shields.io/github/downloads/ffizer/ffizer/total.svg)

ffizer is a files and folders initializer / generator. Create any kind (or part) of project from template.

keywords: file generator, project template, project scaffolding, quickstart, project initializer, project skeleton

<!-- TOC -->

- [ffizer](#ffizer)
  - [Motivations](#motivations)
    - [Main features](#main-features)
    - [Sub features](#sub-features)
  - [Limitations](#limitations)
  - [Usages](#usages)
    - [Install](#install)
      - [via homebrew](#via-homebrew)
      - [via github releases](#via-github-releases)
      - [via cargo](#via-cargo)
    - [Run](#run)
      - [Self upgrade the executable](#self-upgrade-the-executable)
      - [Apply a template](#apply-a-template)
    - [Authoring a template](#authoring-a-template)
  - [Templates](#templates)
  - [Build](#build)
  - [Alternatives](#alternatives)
    - [Generic](#generic)
    - [Specialized](#specialized)

<!-- /TOC -->

<a id="markdown-motivations" name="motivations"></a>
## Motivations

<a id="markdown-main-features" name="main-features"></a>
### Main features

- [X] project generator as a standalone executable (no shared/system dependencies (so no python + pip + ...))
- [X] a simple and generic project template (no specialisation to one ecosystem)
- [X] template as simple as possible, like a
  - [X] copy or clone with file/folder renames without overwrite
  - [X] few search and replace into file
- [X] template hosted as a local folder on the file system
- [X] template hosted as a git repository on any host (not only public github)
  - [X] at root of the repository
  - [X] in subfolder of the repository
  - [X] in any revision (branch, tag, commit)
- [X] template composed of other template
  - [X] composite template are regular standalone template
  - [X] composite template can be apply at root folder
- [X] a fast enough project generator

<a id="markdown-sub-features" name="sub-features"></a>
### Sub features

- [X] dry mode (usefull to test)
- [ ] chain commands (eg: 'git init') (like a post-hook)
  - [ ] raw command
- [ ] composite template include under conditions
- [X] file / folder ignored under conditions (ignores'item in ffizer.yaml are defined as handlerbar expression)
- [X] handlebars helpers
  - [X] transform strings (toUpperCase, toLowerCase, Capitelize,...)
  - [X] render content of GET url
  - [X] render content from https://gitignore.io
  - [ ] suggestions welcomes ;-)
- [ ] ability to update / diff / overwrite existing file

<a id="markdown-limitations" name="limitations"></a>
## Limitations

Some of the following limitations could change in the future (depends on gain/loss):

- no conditionals file or folder creation
- no framework X dedicated features
- no plugin and not extensible (without change the code)
- handlebars is the only template language supported (support for other is welcome)

<a id="markdown-usages" name="usages"></a>
## Usages

<a id="markdown-install" name="install"></a>
### Install

```sh
curl https://raw.githubusercontent.com/ffizer/ffizer/master/scripts/getLatest.sh | sh`
```

<a id="markdown-via-homebrew" name="via-homebrew"></a>
#### via homebrew

```sh
brew tap ffizer/ffizer
brew install ffizer-bin
ffizer upgrade
```

<a id="markdown-via-github-releases" name="via-github-releases"></a>
#### via github releases

Download the binary for your platform from [github releases](https://github.com/ffizer/ffizer/releases), then unarchive it and place it your PATH.

<a id="markdown-via-cargo" name="via-cargo"></a>
#### via cargo

```sh
cargo install ffizer
```

<a id="markdown-run" name="run"></a>
### Run

```txt
➜  ffizer --help

ffizer 0.10.0
https://github.com/ffizer/ffizer
ffizer is a files and folders initializer / generator. Create any kind (or part) of project from template.

USAGE:
    ffizer [FLAGS] <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
    -v, --verbose    Verbose mode (-v, -vv (very verbose / level debug), -vvv) print on stderr

SUBCOMMANDS:
    apply      Apply a template into a target directory
    help       Prints this message or the help of the given subcommand(s)
    upgrade    Self upgrade ffizer executable
```

<a id="markdown-self-upgrade-the-executable" name="self-upgrade-the-executable"></a>
#### Self upgrade the executable

```sh
➜  ffizer upgrade --help

ffizer-upgrade 0.10.0
https://github.com/ffizer/ffizer
Self upgrade ffizer executable

USAGE:
    ffizer upgrade

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
```

<a id="markdown-apply-a-template" name="apply-a-template"></a>
#### Apply a template

```sh
➜  ffizer apply --help

ffizer-apply 0.10.0
https://github.com/ffizer/ffizer
Apply a template into a target directory

USAGE:
    ffizer apply [FLAGS] [OPTIONS] --destination <dst_folder> --source <uri>

FLAGS:
    -h, --help                      Prints help information
        --offline                   in offline, only local templates or cached templates are used
    -V, --version                   Prints version information
        --x-always_default_value    should not ask for valiables values, always use defautl value or empty
                                    (experimental)

OPTIONS:
        --confirm <confirm>               ask confirmation 'never' or 'always' [default: never]
    -d, --destination <dst_folder>        destination folder (created if doesn't exist)
        --rev <rev>                       git revision of the template [default: master]
        --source-subfolder <subfolder>    path of the folder under the source uri to use for template
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
  
<a id="markdown-authoring-a-template" name="authoring-a-template"></a>
### Authoring a template

see [Template Authoring - ffizer](https://ffizer.github.io/ffizer/book/template_authoring.html) *WIP*

<a id="markdown-templates" name="templates"></a>
## Templates

- [ffizer/templates_default: the default collections of templates for ffizer](https://github.com/ffizer/templates_default) (WIP)
- [davidB31 / cg-starter-multi-rust · GitLab](https://gitlab.com/davidB31/cg-starter-multi-rust) Project template for Multi-Bot in Rust on CodinGame.
- samples (used for test, demo)
templates_default)
  - [test_1](tests/test_1/template)
  - [test_2](tests/test_2/template) (demo of usage of gitignore.io)
  - [ffizer/template_sample: a simple template for ffizer used for demo and test](https://github.com/ffizer/template_sample)

<a id="markdown-build" name="build"></a>
## Build

```sh
cargo install cargo-make --force
cargo make ci-flow
```

<a id="markdown-alternatives" name="alternatives"></a>
## Alternatives

<a id="markdown-generic" name="generic"></a>
### Generic

- [Cookiecutter](https://cookiecutter.readthedocs.io/), lot of templates, require python + pip + install dependencies on system (automatic)
- [Cookiecutter — Similar projects](https://cookiecutter.readthedocs.io/en/latest/readme.html#similar-projects)
- [sethyuan/fgen](https://github.com/sethyuan/fgen): A file generator library to be used to generate project structures, file templates and/or snippets. Templates are based on mustache. require nodejs
- [project_init](https://crates.io/crates/project_init) in rust, use mustache for templating but I have some issues with it (project template creation not obvious, github only) I could contributes but I have incompatible requirements.
- [skeleton](https://crates.io/crates/skeleton), good idea but no template file, more like a script.
- [porteurbars](https://crates.io/crates/porteurbars), very similar but I discovered it too late.

<a id="markdown-specialized" name="specialized"></a>
### Specialized

specialized to a platform, build tool,...

- [The web's scaffolding tool for modern webapps | Yeoman](http://yeoman.io/), nodejs ecosystem
- [JHipster - Generate your Spring Boot + Angular/React applications!](https://www.jhipster.tech/) require java, dedicated to java web ecosystem, optionnated template (not generic)
- [Giter8](http://www.foundweekends.org/giter8/) require java + [Conscript](http://www.foundweekends.org/conscript/index.html)
- [Typesafe activator](https://developer.lightbend.com/start/), require java, target scala ecosystem
- [Maven – Archetypes](https://maven.apache.org/guides/introduction/introduction-to-archetypes.html) require java + maven, target maven ecosystem
- [cargo-generate](https://github.com/ashleygwilliams/cargo-generate), limited capabilities, target rust/cargo ecosystem
