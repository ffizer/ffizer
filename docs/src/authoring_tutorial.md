# Tutorial: How to create a template for ffizer <!-- omit in toc -->

- [Begin with a existing sample](#begin-with-a-existing-sample)
- [Clean the content of `my-templates` folder](#clean-the-content-of-my-templates-folder)
- [Test the template](#test-the-template)
- [Enable git versionning](#enable-git-versionning)
- [Publish on remote git repository](#publish-on-remote-git-repository)
- [Parametrize the template with variables](#parametrize-the-template-with-variables)
- [Compose with an external template](#compose-with-an-external-template)
- [Next](#next)

## Begin with a existing sample

Create the folder for the template by copying the existing folder (project)
that you would like to generate with the future template.

```sh
cp -R my-existing-project my-template
cd my-template
```

If you don't have an existing sample you can start with the following code,
just to have something 😉.

```sh
mkdir my-template
cd my-template

cat >README.md <<EOF
Hello World
EOF
```

## Clean the content of `my-templates` folder

- remove files and folder that you don't want to be part of the template
- maybe add  missing files and folder

## Test the template

At this point you have a valid template but it's also a sample of what the
template can produce. Use it to create a sample that can show a possible output,
and a test to validate what ffizer + `my-template` can generate.

```sh
# create the folder for samples (the name of folder should be `.ffizer.samples.d`)
mkdir .ffizer.samples.d

# copy the current of my-template as a sample named `my-sample`
# except .ffizer.samples.d (its why we use `rsync` instead of `cp -R`)
rsync -rv --exclude=.ffizer.samples.d . .ffizer.samples.d/my-sample.expected

# test that ffizer is able to generate every samples under `.ffizer.samples.d`
# from `my-templates`
ffizer test-samples --source .
```

## Enable git versionning

```sh
git init
```

A new folder `.git` was created and it should not be part of the template.
To ignore this folder, you need to add a configuration for ffizer with this information.

```sh
cat >.ffizer.yaml <<EOF
ignores:
  - .git # exclude .git of the template host

EOF
```

*An other alternatives would be to move the template, but not the sample under a `template` folder and to configure `use_template_dir: true`*

Now you can add test again, and commit.

```sh
ffizer test-samples --source .
git add .
git commit -m ":tada: initialize the template"
```

## Publish on remote git repository

ffizer support templates hosted on local file system or on remote git repositories
(accessible via ssh or https). To share your template you can create and push
your local repository to your favorite host (github, gitlab, bitbucket,...)
following the same workflow than for any git repository
(instructions are provided by the hosting service).

```sh
# adapt the following parameters for your case
git remote add origin git@github.com:xxxx/my_template.git
git push -u origin main

# you can also test a remote template
ffizer test-samples --source git@github.com:xxxx/my_template.git
```

## Parametrize the template with variables

The interest of a template is to be more than a `cp -R`, or a `git clone`.
The first expected feature is to be able to render some parametrized content.
ffizer renders only files with `.ffizer.hbs` as part of the filename (it could be at end or before final extension),
the part `.ffizer.hbs` will be removed from the final name after rendering.
[Handlebars](https://handlebarsjs.com/guide/) syntax is used for template string.

```sh
# adapt to your case
mv README.md README.ffizer.hbs.md
cat >README.ffizer.hbs.md <<EOF
Hello {{env_var "USERNAME"}}
EOF
```

At render time, the content between `{{` and  `}}` is evaluated, the sample bellow it call the function `env_var` with the parameter `"USERNAME"`, this function is [built-in](./builtin_functions.md) into ffizer.

If you run the test, an error should be raised, because the rendered content doesn't match the sample.

```sh
> ffizer test-samples --source .

Configure variables



Plan to execute

   - make dir        my-sample
   - add file         └─README.md
Nov 08 17:54:38.837 WARN check failed Differences: [
    EntryDiff {
        expect_base_path: "/home/david/tmp/my-template/.ffizer.samples.d/my-sample.expected",
        actual_base_path: "/tmp/.tmpROuSyU/my-sample",
        relative_path: "README.md",
        difference: StringContent {
            expect: "Hello World\n",
            actual: "Hello david\n",
        },
    },
], sample: my-sample, sub-cmd: test-samples
Nov 08 17:54:38.837 ERRO cmd: CliOpts {
    verbose: 0,
    cmd: TestSamples(
        TestSamplesOpts {
            src: SourceLoc {
                uri: SourceUri {
                    raw: ".",
                    path: ".",
                    host: None,
                },
                rev: "master",
                subfolder: None,
            },
            offline: false,
        },
    ),
}
Nov 08 17:54:38.837 ERRO failed: TestSamplesFailed
```

To fix this issue the better is to define a variable with a value that could be provided by the template user and with the default value define like previously.

```sh
cat >.ffizer.yaml <<EOF
variables:
  - name: author_name
    default_value: '{{ env_var "USERNAME" }}'

ignores:
  - .git # exclude .git of the template host

EOF

cat >README.ffizer.hbs.md <<EOF
Hello {{ author_name }}
EOF
```

At this point if we rerun the test, we will have the same result, because test run `ffizer apply` with the option `--no-interaction` that imply using the default value of a variable. So we need to modify the test to force the value of the variable like a user will do by providing a response to the interactive ask of the value for `author_name` or by using the option `--variables`.

```sh
# setup a configuration for the test sample `my-sample`
cat >.ffizer.samples.d/my-sample.cfg.yaml <<EOF
apply_args:
  - --variables
  - author_name=World

EOF

ffizer test-samples --source .
```

Test is back to OK, with your parametrized template.

File with `.ffizer.hbs` are not the only place where you can use handlebars rendering. It can also be used in file or foldername (see [How to...](./how_to.md)) and into some part of the configuration file `.ffizer.yaml` (see [Configuration](./template_configuration.md)).

## Compose with an external template

To Avoid to duplicate content and configuration between template, you can import the content of one or several other templates into your template. In this tutorial, you will add a template that generate `.gitignore` files from `.gitignore.io` (now branded as part of topcal).

To that you should add a section `imports`:

```sh
cat >.ffizer.yaml <<EOF
variables:
  - name: author_name
    default_value: '{{ env_var "USERNAME" }}'

ignores:
  - .git # exclude .git of the template host

imports:
  - uri: "https://github.com/ffizer/templates_default.git"
    rev: "master"
    subfolder: "gitignore_io"

EOF
```

- The `gitignore_io` template is not hosted at a root level the git repository so you should specify the subfolder (like from the options of `ffizer apply`).
- The uri can in the format `git@github....` or `https://github.com/...`

If you run `ffizer test-samples --source .`, you will have an error because the file `.gitignore` does not exist in the expected of `my-sample`. If you would like to see the generated file, the simplest way is to apply the template on a local folder (because test-samples remove files and folders created during the test). It is also the opportunity to try your template like a final user.

```sh
ffizer apply -s . -d ../my-template-t1
```

You can notice the following points:

- the imported template ask for variable's value
- the ask is after your template's variables
- the ask is not the variable name but a sentence

because the `.ffizer.yaml` of the gitignore_io template includes

```yaml
variables:
  - name: gitignore_what
    default_value: git,visualstudiocode
    ask: Create useful .gitignore (via gitignore.io) for
```

In this tutorial, you will override the default value of `gitignore_what` to be only `git`

```sh
cat >.ffizer.yaml <<EOF
variables:
  - name: author_name
    default_value: '{{ env_var "USERNAME" }}'
  - name: gitignore_what
    default_value: git
    ask: Create useful .gitignore (via gitignore.io) for

ignores:
  - .git # exclude .git of the template host

imports:
  - uri: "https://github.com/ffizer/templates_default.git"
    rev: "master"
    subfolder: "gitignore_io"

EOF
```

And you will also update the `my-sample.expected`.

```sh
ffizer apply -s . -d ../my-template-t2
cp ../my-template-t2/.gitignore .ffizer.samples.d/my-sample.expected
ffizer test-samples --source .
```

You can now remove the result of your previous manual run.

```sh
rm -Rf ../my-template-t*
```

## Next

It's the end of this tutorial but I hope not the end of your journey with ffizer.
Now you know the basics to create, to test, to parametrize and to publish
a template. You can take a look to existing templates and
you can continue to read the ffizer's book, you could learn how to customize ask of value for variable, how to display selection,
how to display a message, how to run command, how to read value of variable from existing file in the project,...

I forgot you can also continue to work with your current template:

- publish / share the new revision of your template
- add parameters
- extract template from your template or other stuff
- compose with other existing templates
- improve the user experience on update
- ...
