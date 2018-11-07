# ffizer

WIP

ffizer is a files and folders generator / initializer. It was created to bootstrap any kind of project.

keywords: file generator, project template, project scaffolding, quickstart, project initializer

## Motivations

### Main features

[ ] project generator as a standalone executable (no shared/system dependencies (so no python + pip + ...))
[ ] a simple and generic project template (no specialisation to one ecosystem)
[ ] template as simple as possible, like a
  [ ] copy or clone with file/folder renames without overwrite
  [ ] few search and replace into file
  [ ] chain commands (eg: 'git init') (like a post-hook)
[ ] template hosted as a local folder on the file system
[ ] template hosted as a (top) git repository on any host (not only public github)
[ ] a fast project generator

### Sub features

[ ] dry mode (usefull to test)

## Limitations

Some of the following limitations could change in the future (depends on gain/loss):

- no conditionnals file or folder creation
- no update of existing file or folder
- no specials features
- no plugin and not extensible (without change the code)

## Usages

### Install

### Run

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
# - the file should have the .ffizer.hbs extention,
# - the extention .ffizer.hbs is removed from the generated filename
# - [Handlebars templating language](https://handlebarsjs.com/)
cat > file1.txt.ffizer.hbs <<EOF
I'm file1.txt of {{ project }}.
EOF
```

The minimal template is an empty dir.

## Build

## Alternatives

### Generic

- [Cookiecutter](https://cookiecutter.readthedocs.io/), lot of templates require python + pip + install dependencies (automatic)
- [Cookiecutter — Similar projects](https://cookiecutter.readthedocs.io/en/latest/readme.html#similar-projects)
- [sethyuan/ffizer: A file generator library to be used to generate project structures, file templates and/or snippets. Templates are based on mustache.](https://github.com/sethyuan/ffizer)

### Specialized

specilazed to a platform, build tool,...

- [The web's scaffolding tool for modern webapps | Yeoman](http://yeoman.io/), nodejs ecosystem
- [JHipster - Generate your Spring Boot + Angular/React applications!](https://www.jhipster.tech/) require java, dedicated to java web ecosystem, optionnated template (not generic)
- [Giter8](http://www.foundweekends.org/giter8/) require java + sbt
- [Typesafe activator](https://developer.lightbend.com/start/), require java, scala ecosystem
- [Maven – Archetypes](https://maven.apache.org/guides/introduction/introduction-to-archetypes.html) require java + maven, maven ecosystem
