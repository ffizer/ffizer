# Configuration

The configuration is:

- optionnal
- stored into a yaml file named `.ffizer.yaml` at the root of the template.
- sections (top level entry) of the yaml are optionnals

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
- `default_value`: a suggested value, the value is a string and support hbs templating
- `ask`: the sentence use to prompt user to set the value of the variable

Variables definition are prompt in the order of the list, and with the prompt defined by `ask` (if defined, else `name`)

```yaml
variables:
  - name: project_name
    default_value: "{{ file_name ffizer_dst_folder }}"
```

### ignores

### imports
