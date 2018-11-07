# fgen

WIP

fgen is a files and folder generator. It was created to bootstrap any kind of project.

## Motivations

### Main features

[ ] project generator as a standalone executable (no shared/system dependencies (so no python + pip + ...))
[ ] a simple and generic project template (no specialisation to one ecosystem)
[ ] template as simple as possible, like a
  [ ] copy or clone with file/folder renames
  [ ] few search and replace into file
  [ ] chain commands (eg: 'git init') (like a post-hook)
[ ] template hosted as a local folder on the file system
[ ] template hosted as a (top) git repository on any host (not only public github)
[ ] a fast project generator

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

( from scratch without fgen ;-) )

```sh
# create the folder with the template
mkdir my-template
cd my-template

# add file that will be copied as is
cat > file0.txt <<EOF
I'm file0.
EOF

# add a template file that will be "rendered" by the handlebars engine
# - the file should have the .fgen.hb extention,
# - the extention .fgen.hb is removed from the generated filename
cat > file1.txt.fgen.hb <<EOF
I'm file0.
EOF
```

The minimal template is an empty dir.

## Build

## Alternatives

- [Cookiecutter](https://cookiecutter.readthedocs.io/), lot of templates
- [Cookiecutter — Similar projects](https://cookiecutter.readthedocs.io/en/latest/readme.html#similar-projects)
- yeoman
- hipsterj
- giter8
- activator
