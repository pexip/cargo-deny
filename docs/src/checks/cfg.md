# config

The top level config for cargo-deny, by default called `deny.toml`.

## Example - cargo-deny's own configuration

```ini
{{#include ../../../deny.toml}}
```

### The `targets` field (optional)

By default, cargo-deny will consider every single crate that is resolved by cargo, including target specific dependencies eg

```ini
[target.x86_64-pc-windows-msvc.dependencies]
winapi = "0.3.8"

[target.'cfg(target_os = "fuchsia")'.dependencies]
fuchsia-cprng = "0.1.1"
```

But unless you are actually targeting `x86_64-fuchsia` or `aarch64-fuchsia`, the `fuchsia-cprng` is never actually going to be compiled or linked into your project, so checking it is pointless for you.

The `targets` field allows you to specify one or more targets which you **actually** build for. Every dependency link to a crate is checked against this list, and if none of the listed targets satisfy the target constraint, the dependency link is ignored. If a crate has no dependency links to it, it is not included into the crate graph that the checks are executed against.

#### The `triple` field

The [target triple](https://forge.rust-lang.org/release/platform-support.html) for the target you wish to filter target specific dependencies with. If the target triple specified is **not** one of the targets builtin to `rustc`, the configuration check for that target will be limited to only the raw `[target.<target-triple>.dependencies]` style of target configuration, as `cfg()` expressions require us to know the details about the target.

#### The `exclude` field (optional)

Just as with the [`--exclude`](../cli/common.md#--exclude) command line option, this field allows you to specify one or more [Package ID specifications](https://doc.rust-lang.org/cargo/commands/cargo-pkgid.html) that will cause the crate(s) in question to be excluded from the crate graph that is used for the operation you are performing.

Note that excluding a crate is recursive, if any of its transitive dependencies are only referenced via the excluded crate, they will also be excluded from the crate graph.

#### The `features` field (optional)

Rust `cfg()` expressions support the [`target_feature = "feature-name"`](https://doc.rust-lang.org/reference/attributes/codegen.html#the-target_feature-attribute) predicate, but at the moment, the only way to actually pass them when compiling is to use the `RUSTFLAGS` environment variable. The `features` field allows you to specify 1 or more `target_feature`s you plan to build with, for a particular target triple. At the time of this writing, cargo-deny does not attempt to validate that the features you specify are actually valid for the target triple, but this is [planned](https://github.com/EmbarkStudios/cfg-expr/issues/1).

### The `[licenses]` section

See the [licenses config](licenses/cfg.html) for more info.

### The `[bans]` section

See the [bans config](bans/cfg.html) for more info.

### The `[advisories]` section

See the [advisories config](advisories/cfg.html) for more info.

### The `[sources]` section

See the [sources config](sources/cfg.html) for more info.
