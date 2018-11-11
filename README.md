# ffizer

<!-- copy badges from:
- [repostatus.org](https://www.repostatus.org/#active)
- travis.com
- license -->

[![Project Status: Concept – Minimal or no implementation has been done yet, or the repository is only intended to be a limited example, demo, or proof-of-concept.](https://www.repostatus.org/badges/latest/concept.svg)](https://www.repostatus.org/#concept)
[![License: CC0-1.0](https://img.shields.io/badge/License-CC0%201.0-lightgrey.svg)](http://creativecommons.org/publicdomain/zero/1.0/)
[![Build Status](https://travis-ci.com/davidB/ffizer.svg?branch=master)](https://travis-ci.com/davidB/ffizer)

ffizer is a files and folders initializer / generator. Create any kind (or part) of project from template.

keywords: file generator, project template, project scaffolding, quickstart, project initializer, project skeleton

## Motivations

### Main features

- [X] project generator as a standalone executable (no shared/system dependencies (so no python + pip + ...))
- [X] a simple and generic project template (no specialisation to one ecosystem)
- [ ] template as simple as possible, like a
  - [ ] copy or clone with file/folder renames without overwrite
  - [ ] few search and replace into file
  - [ ] chain commands (eg: 'git init') (like a post-hook)
- [X] template hosted as a local folder on the file system
- [ ] template hosted as a (top) git repository on any host (not only public github)
- [ ] a fast enough project generator

### Sub features

- [X] dry mode (usefull to test)
- [ ] chain template generation because fragment of templates can be commons

## Limitations

Some of the following limitations could change in the future (depends on gain/loss):

- no conditionnals file or folder creation
- no update of existing file or folder
- no specials features
- no plugin and not extensible (without change the code)
- handlebars is the only template language supported (support for other is welcome)

## Usages

### Install

### Run

```sh
ffizer 0.1.0
davidB
ffizer is a files and folders generator / initializer. Create any kind of project from template.

USAGE:
    ffizer [FLAGS] --destination <folder> --source <uri>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
    -v, --verbose    Verbose mode (-v, -vv (very verbose / level debug), -vvv) print on stderr

OPTIONS:
    -d, --destination <folder>    destination folder (created if doesn't exist)
    -s, --source <uri>            uri / path of the template
```

### Create your first template

( from scratch without ffizer ;-) )

```sh
# create the folder with the template
mkdir my-template
cd my-template

# add file that will be copied as is
cat > file0.txt <<EOF
I'm file0.
EOF

# add a template file that will be "rendered" by the handlebars engine
# - the file should have the .ffizer.hbs extension,
# - the extension .ffizer.hbs is removed from the generated filename
# - [Handlebars templating language](https://handlebarsjs.com/)
cat > file1.txt.ffizer.hbs <<EOF
I'm file1.txt of {{ project }}.
EOF
```

The minimal template is an empty dir.

## Build

## Alternatives

### Generic

- [Cookiecutter](https://cookiecutter.readthedocs.io/), lot of templates, require python + pip + install dependencies on system (automatic)
- [Cookiecutter — Similar projects](https://cookiecutter.readthedocs.io/en/latest/readme.html#similar-projects)
- [sethyuan/fgen](https://github.com/sethyuan/fgen): A file generator library to be used to generate project structures, file templates and/or snippets. Templates are based on mustache. require nodejs
- [project_init](https://crates.io/crates/project_init) in rust, use mustache for templating but I have some issues with it (project template creation not obvious, github only, plus few bug) I could contributes but I have incompatible requirements (and would like to create my own since a long time).
- [skeleton](https://crates.io/crates/skeleton), good idea but no template file, more like a script.
- [porteurbars](https://crates.io/crates/porteurbars), very similar but I discover it too late.

### Specialized

specilazed to a platform, build tool,...

- [The web's scaffolding tool for modern webapps | Yeoman](http://yeoman.io/), nodejs ecosystem
- [JHipster - Generate your Spring Boot + Angular/React applications!](https://www.jhipster.tech/) require java, dedicated to java web ecosystem, optionnated template (not generic)
- [Giter8](http://www.foundweekends.org/giter8/) require java + [Conscript](http://www.foundweekends.org/conscript/index.html)
- [Typesafe activator](https://developer.lightbend.com/start/), require java, target scala ecosystem
- [Maven – Archetypes](https://maven.apache.org/guides/introduction/introduction-to-archetypes.html) require java + maven, target maven ecosystem
- [cargo-generate](https://github.com/ashleygwilliams/cargo-generate), limited capabilities, target rust/cargo ecosystem
