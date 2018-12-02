# ffizer

<!-- copy badges from:
- [repostatus.org](https://www.repostatus.org/#active)
- [Shields.io: Quality metadata badges for open source projects](https://shields.io/#/)
-->

[![Crates.io](https://img.shields.io/crates/l/ffizer.svg)](http://creativecommons.org/publicdomain/zero/1.0/)
[![Crates.io](https://img.shields.io/crates/v/ffizer.svg)](https://crates.io/crates/ffizer)

[![Project Status: WIP – Initial development is in progress, but there has not yet been a stable, usable release suitable for the public.](https://www.repostatus.org/badges/latest/wip.svg)](https://www.repostatus.org/#wip)
[![Build Status](https://travis-ci.com/davidB/ffizer.svg?branch=master)](https://travis-ci.com/davidB/ffizer)

[![Crates.io](https://img.shields.io/crates/d/ffizer.svg)](https://crates.io/crates/ffizer)
![GitHub All Releases](https://img.shields.io/github/downloads/davidB/ffizer/total.svg)

ffizer is a files and folders initializer / generator. Create any kind (or part) of project from template.

keywords: file generator, project template, project scaffolding, quickstart, project initializer, project skeleton

<!-- TOC -->

- [Motivations](#motivations)
    - [Main features](#main-features)
    - [Sub features](#sub-features)
- [Limitations](#limitations)
- [Usages](#usages)
    - [Install](#install)
        - [via github releases](#via-github-releases)
        - [via cargo](#via-cargo)
    - [Run](#run)
    - [Create a template](#create-a-template)
        - [Rules](#rules)
        - [A 5 minutes tutorial](#a-5-minutes-tutorial)
        - [Pre-defined variables](#pre-defined-variables)
        - [Template Helpers / Functions](#template-helpers--functions)
            - [String transformation](#string-transformation)
            - [Http content](#http-content)
            - [Path extraction](#path-extraction)
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
- [ ] template composed of other template
  - [ ] composite template are regular standalone template
  - [ ] composite template can be apply at root folder
- [X] a fast enough project generator

<a id="markdown-sub-features" name="sub-features"></a>
### Sub features

- [X] dry mode (usefull to test)
- [ ] chain template generation because fragment of templates can be commons
- [ ] chain commands (eg: 'git init') (like a post-hook)
  - [ ] raw command
- [ ] composite template can be apply at root folder
- [ ] composite template include under conditions
- [X] file / folder ignored under conditions (ignores'item in ffizer.yaml are defined as handlerbar expression)
- [X] handlebars helpers
  - [X] transform strings (toUpperCase, toLowerCase, Capitelize,...)
  - [X] render content of GET url
  - [X] render content from https://gitignore.io
  - [ ] suggestions welcomes ;-)

<a id="markdown-limitations" name="limitations"></a>
## Limitations

Some of the following limitations could change in the future (depends on gain/loss):

- no conditionnals file or folder creation
- no update of existing file or folder
- no specials features
- no plugin and not extensible (without change the code)
- handlebars is the only template language supported (support for other is welcome)

<a id="markdown-usages" name="usages"></a>
## Usages

<a id="markdown-install" name="install"></a>
### Install

<a id="markdown-via-github-releases" name="via-github-releases"></a>
#### via github releases

Download the binary for your platform from [github releases](https://github.com/davidB/ffizer/releases), then unarchive it and place it your PATH.

<a id="markdown-via-cargo" name="via-cargo"></a>
#### via cargo

```sh
cargo install ffizer
```

<a id="markdown-run" name="run"></a>
### Run

```txt
ffizer 0.7.0
davidB
ffizer is a files and folders initializer / generator. Create any kind (or part) of project from template.

USAGE:
    ffizer [FLAGS] [OPTIONS] --destination <dst_folder> --source <src_uri>

FLAGS:
    -h, --help                      Prints help information
        --offline                   in offline, only local templates or cached templates are used
    -V, --version                   Prints version information
    -v, --verbose                   Verbose mode (-v, -vv (very verbose / level debug), -vvv) print on stderr
        --x-always_default_value    should not ask for valiables values, always use defautl value or empty
                                    (experimental)

OPTIONS:
        --confirm <confirm>             ask confirmation 'never' or 'always' [default: never]
    -d, --destination <dst_folder>      destination folder (created if doesn't exist)
        --source-folder <src_folder>    path of the folder under the source uri to use for template
        --rev <src_rev>                 git revision of the template [default: master]
    -s, --source <src_uri>              uri / path of the template
```

- use a local folder as template
    ```sh
    ffizer --source $HOME/my_templates/tmpl0 --destination my_project
    ```
- use a remote git repository as template
    ```sh
    ffizer --source https://github.com/davidB/ffizer_demo_template.git --destination my_project
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
  
<a id="markdown-create-a-template" name="create-a-template"></a>
### Create a template

<a id="markdown-rules" name="rules"></a>
#### Rules

- The minimal template is an empty dir.
- a sample template and its expected output (on empty folder) is available at [tests/test_1](tests/test_1).
- file priority (what file will be used if they have the same destination path)

  ```txt
  existing file
  file with source extension .ffizer.hbs (and no {{...}} in the source file path)
  file with identical source file name (and extension)
  file with {{...}} in the source file path
  ```

<a id="markdown-a-5-minutes-tutorial" name="a-5-minutes-tutorial"></a>
#### A 5 minutes tutorial

1. create the folder with the template
    ```sh
    mkdir my-template
    cd my-template
    ```

1. add file that will be copied as is
    ```sh
    cat > file0.txt <<EOF
    I am file0.
    EOF
    ```

1. add a template file that will be "rendered" by the handlebars engine

    - the file should have the .ffizer.hbs extension,
    - the extension .ffizer.hbs is removed from the generated filename
    - [Handlebars templating language](https://handlebarsjs.com/)
  
    ```sh
    cat > file1.txt.ffizer.hbs <<EOF
    I am file1.txt of {{ project }}.
    EOF
    ```
1. add a ffizer configuration file (.ffizer.yaml)
    - to list variables
    - to list pattern to ignore

    ```yaml
    variables:
      - name: project
        default_value: my-project

    ignores:
      - .git # exclude .git of the template host
    ```
1. add a file with a name that will be "rendered" by the handlebars engine

    - the file should have {{ variable }},
    - [Handlebars templating language](https://handlebarsjs.com/)

    ```sh
    cat > '{{ project }}.txt' <<EOF
    I am a fixed content file with rendered file name.
    EOF
    ```

<a id="markdown-pre-defined-variables" name="pre-defined-variables"></a>
#### Pre-defined variables

Some variables are predefined and they can be used into `ffizer.yaml` into the `ignores` section or `default_value` via handlebars expression.

```yaml
variables:
  - name: project_name
    default_value: "{{ file_name ffizer_dst_folder }}"
```

The predefined variables are:

- `ffizer_dst_folder` contains the value from cli arg `--destination`
- `ffizer_src_rev` contains the value from cli arg `--rev`
- `ffizer_src_uri` contains the value from cli arg `--source`

<a id="markdown-template-helpers--functions" name="template-helpers--functions"></a>
#### Template Helpers / Functions

Helpers extend the template to generate or to transform content.
Few helpers are included, but if you need more helpers, ask via an issue or a PR.

To use an helper:

```handlebars
{{ helper_name argument}}
```

To chain helpers, use parenthesis:

```handlebars
{{ to_upper_case (to_singular "Hello foo-bars") }}
// -> "BAR"
```

see [Handlebars templating language](https://handlebarsjs.com/)

<a id="markdown-string-transformation" name="string-transformation"></a>
##### String transformation

for the same input: "Hello foo-bars"

helper_name | example out
-- | --
to_lower_case | "hello foo-bars"
to_upper_case | "HELLO FOO-BARS"
to_camel_case | "helloFooBars"
to_pascal_case | "HelloFooBars"
to_snake_case | "hello_foo_bars"
to_screaming_snake_case | "HELLO_FOO_BARS"
to_kebab_case | "hello-foo-bars"
to_train_case | "Hello-Foo-Bars"
to_sentence_case | "Hello foo bars"
to_title_case | "Hello Foo Bars"
to_class_case | "HelloFooBar"
to_table_case | "hello_foo_bars"
to_plural | "bars"
to_singular | "bar"

<a id="markdown-http-content" name="http-content"></a>
##### Http content

Helper able to render body response from an http request.

helper_name | usage
-- | --
http_get | http_get "http://hello/..."
gitignore_io | gitignore_io "rust"

<a id="markdown-path-extraction" name="path-extraction"></a>
##### Path extraction

Helper able to extract (or transform) path (defined as string).

for the same input: "/hello/bar/foo.txt"

helper_name | sample output
-- | --
file_name | "foo.txt"
parent | "/hello/bar"
extension | "txt"

<a id="markdown-templates" name="templates"></a>
#### Templates

- samples (used for test, demo)
  - [test_1](tests/test_1/template)
  - [test_2](tests/test_2/template) (demo of usage of gitignore.io)
  - [davidB/ffizer_demo_template: a simple template for ffizer used for demo and test](https://github.com/davidB/ffizer_demo_template)
- [davidB31 / cg-starter-multi-rust · GitLab](https://gitlab.com/davidB31/cg-starter-multi-rust) Project template for Multi-Bot in Rust on CodinGame.


<a id="markdown-build" name="build"></a>
## Build

```sh
cargo test
cargo build --release
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
