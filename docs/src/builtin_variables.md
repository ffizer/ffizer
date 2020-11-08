# The predefined variables

Some variables are predefined and they can be used into `.ffizer.yaml` into by example
the `imports` section or `default_value` via handlebars expression.

- `ffizer_dst_folder` contains the value from cli arg `--destination`
- `ffizer_src_rev` contains the value from cli arg `--rev`
- `ffizer_src_uri` contains the value from cli arg `--source`

The following sample combine a helper function `file_name` with `ffizer_dst_folder`.

```yaml
variables:
  - name: project_name
    default_value: "{{ file_name ffizer_dst_folder }}"
```
