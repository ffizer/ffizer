# A 5 minutes tutorial

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

    - the file should have the **.ffizer.hbs** extension,
    - the extension .ffizer.hbs is removed from the generated filename
    - [Handlebars templating language](https://handlebarsjs.com/)
  
    ```sh
    cat > file1.txt.ffizer.hbs <<EOF
    I am file1.txt of {{ project }}.
    EOF
    ```

1. add a ffizer configuration file (**.ffizer.yaml**)
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
