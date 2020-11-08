# How to... <!-- omit in toc -->

- [How to name a folder as a package name of the project ?](#how-to-name-a-folder-as-a-package-name-of-the-project-)
- [How to display a message after files'generation ?](#how-to-display-a-message-after-filesgeneration-)
- [How to run a set of commands after files'generation ?](#how-to-run-a-set-of-commands-after-filesgeneration-)
- [How to import a sibling template ?](#how-to-import-a-sibling-template-)
- [How to update existing json/yaml/toml content ?](#how-to-update-existing-jsonyamltoml-content-)
- [How to retrieve value from existing json/yaml/toml content ?](#how-to-retrieve-value-from-existing-jsonyamltoml-content-)
- [How to made "ignore files" conditional ?](#how-to-made-ignore-files-conditional-)
- [How to include a `.git` folder as part of the template ?](#how-to-include-a-git-folder-as-part-of-the-template-)
- [How to test my template ?](#how-to-test-my-template-)
- [How to host template on github ?](#how-to-host-template-on-github-)

## How to name a folder as a package name of the project ?

Define a `package_name` variable with a sensible value (eg snake case of the project name).

```yaml
variables:
  - name: project_name
    default_value: "{{ file_name ffizer_dst_folder }}"
  - name: package_name
    default_value: "{{ to_snake_case project_name }}"
```

And use it as name of the folder.

```sh
mkdir 'src/{{ package_name }}`
```

## How to display a message after files'generation ?

in `.ffizer.yaml`

```yaml
scripts:
  - message: |
      Thanks for using my awesome template.
```

## How to run a set of commands after files'generation ?

in `.ffizer.yaml`

```yaml
scripts:
  - cmd: |
      {{#if (eq (env_var "OS") "windows") }}
      echo Hello {{ who }}> file2.txt
      del file_to_delete.txt
      {{else}}
      echo "Hello {{ who }}" > file2.txt
      rm file_to_delete.txt
      {{/if}}
```

- Having several entries under `scripts` with `cmd` is allowed.
- Empty `cmd` after template rendering are ignored.
- Each `cmd` block is displayed to the user to confirm
  if (s)he accepts to run it or not.

## How to import a sibling template ?

If your git repository host several templates, one template can import a sibling
template by specifying the absolute value or by using built-in variables

```yaml
imports:
  - uri: "{{ ffizer_src_uri }}"
    rev: "{{ ffizer_src_rev }}"
    subfolder: "template_2"
```

## How to update existing json/yaml/toml content ?

## How to retrieve value from existing json/yaml/toml content ?

## How to made "ignore files" conditional ?

## How to include a `.git` folder as part of the template ?

## How to test my template ?

## How to host template on github ?
