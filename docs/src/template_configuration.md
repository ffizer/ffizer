# Configuration

The configuration is:

- optional
- stored into a yaml file named `.ffizer.yaml` at the root of the template.
- sections (top level entry) of the yaml are optionals

```yaml
variables:
  - name: project
    default_value: my-project

ignores:
  - .git # exclude .git of the template host
```

## Sections

### variables

List the variables usable into the `.ffizer.hbs` template file.
Variables are defined by:

- `name`: (required) the name of the variable
- `default_value`: a suggested value, the value is a string and support `hbs` templating
- `ask`: the sentence use to prompt user to set the value of the variable

Variables definition are prompt in the order of the list, and with the prompt defined by `ask` (if defined, else `name`)

```yaml
variables:
  - name: project_name
    default_value: "{{ file_name ffizer_dst_folder }}"
```

Every variables are mandatory, to allow empty value `default_value` should be an empty string.

```yaml
  - name: foo
    default_value: ""
```

### ignores

List patterns of file path (relative to root of the template) that should be ignored when search for file to be copied or rendered from the template into the destination.

To ignore .git folder from the template (useful for template hosted on root of a git repository)

```yaml
ignores:
  - .git/*
  - .git # exclude .git of the template host
```

### imports

It is possible to imports templates into a template. It is useful to reuse templates or to compose template from other template.

The `imports` section is a list of templates:

```yaml
imports:
  - uri: "git@github.com:ffizer/templates_default.git"
    rev: "master"
    subfolder: "gitignore_io"
```

The order in the list define:

- the order to ask variables (and to find variables definition): first the variable of the root template, then the variables of the first import, the second import,... then the variables of the first import of the first imports.
- the order of files: first the file of the root template, then the files from the first import,... (same as variable)

<!-- TODO insert a diagram of priority and order -->

The first variable definition found (following the order) is keep. So a higher level variables definition override the lower level. In the example below, the `ask`and the `default_value` override the definition of `gitignore_what` into the imported template.

```yaml
variables:
  - name: gitignore_what
    default_value: rust,git,visualstudiocode
    ask: Create useful .gitignore (via gitignore.io) for

imports:
  - uri: "git@github.com:ffizer/templates_default.git"
    rev: "master"
    subfolder: "gitignore_io"
```

