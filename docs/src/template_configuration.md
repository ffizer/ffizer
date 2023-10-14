# Configuration

The configuration is:

- optional
- Saved within a yaml file named `.ffizer.yaml`, located at the root of the template.
- All sections (top level entries) of the yaml file are optional.

```yaml
# Define variables for the project setup
variables:
  - name: project_name
    default_value: my-project
  - name: Add extra features?
    select_in_values: ['yes', 'no']

# Specify the list of files to exclude during copying or rendering process
ignores:
  - .git # exclude .git of the template host
  - README.md

# scripts to be executed after file generation
scripts:
  - cmd: echo "{{ project_name }} generated!"
    default_confirm_answer: 'yes'
  
# Set to `true` to define the content root as `template` instead of the root folder
use_template_dir: false

# import other templates
imports:
  - uri: "git@github.com:ffizer/templates_default.git"
    rev: "master"
    subfolder: "gitignore_io"
```

## Sections

### variables

List the variables usable into the `.ffizer.hbs` template file.
Variables are defined by:

- `name`: (required) the name of the variable.
- `default_value`: a suggested value, the value is a string and support `hbs` templating.
- `ask`: the sentence use to prompt user to set the value of the variable.
- `hidden`: the variable is not shown to the user, the value is set to default_value. Could be useful to cache shared (structured) value. (default to false)
- `select_in_values`: for non-empty list, ask the user to select a value in the list. The list can be a regular yaml list or a string (evaluated as a yaml list of string). `default_value` could be combined to pre-select a value in the list. After selection a second variable with same name plus suffix `__idx` is set with the index of the selected value in the list.

  ```yaml
          variables:
            - name: k2
              select_in_values:
                - vk21
                - vk22
            - name: k1
              select_in_values: [ "vk11", "vk12" ]
            - name: k3
              select_in_values: '[ "vk31", "vk32" ]'
            - name: k4
              select_in_values: '{{ do_stuff }}'
  ```

Variables definition are prompt in the order of the list, and with the prompt defined by `ask` (if defined, else `name`)

```yaml
variables:
  - name: project_name
    default_value: "{{ file_name ffizer_dst_folder }}"
```

- Every variable is mandatory.
- To use an empty value `default_value` should be an empty string.

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

### scripts

Scripts allows you to run arbitrary commands after generating the files.

```yaml
scripts:
  - cmd: echo "{{ project_name }} generated!"
    default_confirm_answer: 'yes'
  - cmd: |
      {{#if (eq (env_var "OS") "windows") }}
      echo Hello {{ who }}> file2.txt
      del file_to_delete.txt
      {{else}}
      echo "Hello {{ who }}" > file2.txt
      rm file_to_delete.txt
      {{/if}}
  - message: |
      Thanks for using my awesome template.
```

- You can include multiple `cmd` entries under the `scripts` section.
- Any `cmd` that results in an empty string after template rendering will be ignored.
- Each `cmd` instruction will be presented to the user for confirmation before being executed. If using `--no-interaction`, the default answer is used.
- Use the `message` field to print an arbitrary message

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

### use_template_dir

By default, content of the template is mixed with its optional metadata (`.ffizer.yaml`, ...). So it means that if you have a `README.md` both as the template description and as template content (a README.md to generate), you have to follow this layout:

```txt
+- README.md.ffizer.hbs (or README.md.ffizer.raw)
+- README.md
+- .ffizer.yaml
```

And to add into `.ffizer.yaml`

```yaml
ignores:
  - README.md
```

Or you can choose to move the template content under a `template` folder:

```txt
+- template
|  +- README.md.ffizer.hbs (or README.md)
+- README.md
+- .ffizer.yaml
```

And to add into `.ffizer.yaml`

```yaml
use_template_dir: true
```