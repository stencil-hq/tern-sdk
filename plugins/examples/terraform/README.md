# terraform

Reads `terraform plan` and `tofu plan` output as a native view instead of
a wall of text: summary badges ("2 to add", "1 to change", "2 to destroy",
output changes, errors, warnings), diagnostics with their source excerpt,
and the resources grouped by action, the destructive groups first. A plan
that destroys or replaces anything gets a red rule down its side.

- **Click a resource** to copy its address (`module.db.aws_db_instance.main`)
  for `-target=`, `terraform state show` or a review comment.
- **Raw** in the block's header still shows the original output.
- Plans the lens cannot read (`-json`, `-help`, unknown output) stay raw.

It claims `terraform plan …` and `tofu plan …`, also after `-chdir=DIR`.
It does not claim `apply`: apply stops at an interactive "Enter a value:"
prompt that the native view would hide.

## Install

```sh
git clone https://github.com/stencil-hq/tern-sdk
tern plugin install tern-sdk/plugins/examples/terraform
```

or, to load it in place and reload on every save:

```sh
tern plugin link tern-sdk/plugins/examples/terraform
```

Command lenses must be on (the `command_lenses` setting, on by default), and
the shell needs Tern's shell integration, which reports each command line.

Walkthrough: <https://docs.stencil.so/tern/examples/terraform.html>.
