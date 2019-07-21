# Template Helpers / Functions

Helpers extend the template to generate or to transform content.

The helpers from [handlebars_misc_helpers](https://crates.io/crates/handlebars_misc_helpers) are included. But if you need more helpers, ask via an issue or a PR.

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

## String transformation

for the same input: `"Hello foo-bars"`

helper_name | example out
-- | --
`to_lower_case` | `"hello foo-bars"`
`to_upper_case` | `"HELLO FOO-BARS"`
`to_camel_case` | `"helloFooBars"`
`to_pascal_case` | `"HelloFooBars"`
`to_snake_case` | `"hello_foo_bars"`
`to_screaming_snake_case` | `"HELLO_FOO_BARS"`
`to_kebab_case` | `"hello-foo-bars"`
`to_train_case` | `"Hello-Foo-Bars"`
`to_sentence_case` | `"Hello foo bars"`
`to_title_case` | `"Hello Foo Bars"`
`to_class_case` | `"HelloFooBar"`
`to_table_case` | `"hello_foo_bars"`
`to_plural` | `"bars"`
`to_singular` | `"bar"`

## Http content

Helper able to render body response from an http request.

helper_name | usage
-- | --
`http_get` | `http_get "http://hello/..."`
`gitignore_io` | `gitignore_io "rust"`

## Path extraction

Helper able to extract (or transform) path (defined as string).

for the same input: `"/hello/bar/foo.txt"`

helper_name | sample output
-- | --
file_name | `"foo.txt"`
parent | `"/hello/bar"`
extension | `"txt"`

## Environment variable

Helper able to get environment variables.

helper_name | usage
-- | --
env_var | `env_var "HOME"`
