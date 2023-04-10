# The predefined variables

Some variables are predefined and they can be used into `.ffizer.yaml` into by example
the `imports` section or `default_value` via handlebars expression.

| variable               | value of                                               |
| ---------------------- | ------------------------------------------------------ |
| `ffizer_dst_folder`    | cli arg `--destination` as string                      |
| `ffizer_src_rev`       | cli arg `--rev` as string (could be null)              |
| `ffizer_src_uri`       | cli arg `--source` as string                           |
| `ffizer_src_subfolder` | cli arg `--source-subfolder` as string (could be null) |
| `ffizer_version`       | the current version of ffizer as string                |

The following sample combine a helper function `file_name` with `ffizer_dst_folder`.

```yaml
variables:
  - name: project_name
    default_value: "{{ file_name ffizer_dst_folder }}"
```
