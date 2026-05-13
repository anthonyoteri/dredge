# dredge

`dredge` is a command-line tool for interacting with the [Docker Registry HTTP API V2](https://distribution.github.io/distribution/spec/api/). It lets you inspect, list, and delete images and tags in any V2-compatible container registry directly from your terminal.

## Features

- List all repositories in a registry catalog
- List all tags for a given image
- Show detailed manifest information for a tagged image
- Delete a tagged image by resolving its digest and removing the manifest
- Verify that a registry endpoint speaks the Docker Distribution API v2

## Installation

Install from [crates.io](https://crates.io/crates/dredge-tool) using Cargo:

```sh
cargo install dredge-tool
```

The installed binary is named `dredge`.

### Prerequisites

- **Rust toolchain** 1.80 or later. Install via [rustup](https://rustup.rs/).
- A running **Docker Registry V2** endpoint. The registry must be accessible over the network from the machine running `dredge`.
- For delete operations, the registry must have storage deletion enabled (see [Deleting a tagged image](#deleting-a-tagged-image)).

## Usage

```
dredge [OPTIONS] <REGISTRY> <COMMAND>
```

### `<REGISTRY>` argument format

The `<REGISTRY>` positional argument accepts any of the following forms:

| Form | Example |
|---|---|
| Hostname | `registry.example.com` |
| Host and port | `registry.example.com:5000` |
| Full URL | `https://registry.example.com:5000` |

When no scheme is provided, `https://` is assumed automatically.

### Global options

| Option | Default | Description |
|---|---|---|
| `--log-level=<LEVEL>` | `info` | Set the log verbosity. Possible values: `trace`, `debug`, `info`, `warn`, `error`, `off`. |
| `-h, --help` | | Print help information. |
| `-V, --version` | | Print version information. |

---

## Subcommands

### Checking the API version

Verify that the registry endpoint implements the Docker Distribution API v2.

```
dredge <REGISTRY> check
```

**Example:**

```sh
dredge registry.example.com check
# Ok
```

---

### Listing repositories (catalog)

Fetch the full list of repositories available in the registry. Handles paginated responses automatically.

```
dredge <REGISTRY> catalog
```

**Example:**

```sh
dredge registry.example.com catalog
# myorg/frontend
# myorg/backend
# myorg/worker
```

---

### Listing tags for an image

Fetch the list of all tags published for a given image. Handles paginated responses automatically.

```
dredge <REGISTRY> tags <NAME>
```

| Argument | Description |
|---|---|
| `<NAME>` | The repository name (e.g. `myorg/backend`). |

**Example:**

```sh
dredge registry.example.com tags myorg/backend
# latest
# v1.0.0
# v1.1.0
# v2.0.0-rc1
```

---

### Showing image details

Show detailed manifest information for a specific tagged image, including the architecture, filesystem layers, digest, and ETag. Output is formatted as YAML.

```
dredge <REGISTRY> show <IMAGE> [TAG]
```

| Argument | Default | Description |
|---|---|---|
| `<IMAGE>` | | The repository name (e.g. `myorg/backend`). |
| `[TAG]` | `latest` | The tag to inspect. Defaults to `latest` if omitted. |

**Example:**

```sh
dredge registry.example.com show myorg/backend v2.0.0-rc1
# name: myorg/backend
# tag: v2.0.0-rc1
# architecture: amd64
# fsLayers:
# - blobSum: sha256:a3ed95caeb02ffe68...
# - blobSum: sha256:7d97e254a0461b0a3...
# digest: sha256:0259571889ac87efbf...
# etag: sha256:0259571889ac87efbf...
```

Omitting the tag inspects `latest`:

```sh
dredge registry.example.com show myorg/backend
```

---

### Deleting a tagged image

Delete a specific tagged image from the registry. The tag is resolved to its content digest, and the manifest is deleted by digest.

```
dredge <REGISTRY> delete <IMAGE> <TAG>
```

| Argument | Description |
|---|---|
| `<IMAGE>` | The repository name (e.g. `myorg/backend`). |
| `<TAG>` | The tag to delete (e.g. `v1.0.0`). |

**Example:**

```sh
dredge registry.example.com delete myorg/backend v1.0.0
```

> **Note:** This requires the registry to have storage deletion enabled. When
> running a registry container, set the environment variable
> `REGISTRY_STORAGE_DELETE_ENABLED=true`. If deletion is not enabled, the
> registry will return a `MethodNotAllowed` error.

> **Note:** This operation removes only the manifest referenced by the given
> tag. Unreferenced layer blobs (orphaned digests) are not removed
> automatically. Run the registry's garbage collector separately to reclaim
> storage space.

---

## Configuration

There is no configuration file. All settings are passed as command-line arguments.

**Enabling verbose logging:**

```sh
dredge --log-level=debug registry.example.com catalog
```

**Silencing all log output:**

```sh
dredge --log-level=off registry.example.com catalog
```

---

## Known Limitations

- **No authentication support.** Registries that require authentication (e.g., Docker Hub, private registries protected by HTTP Basic Auth or token-based auth) are not currently supported. Requests to such registries will fail with an `HTTP Authorization failed` error.
- **Delete only removes the manifest tag, not layer blobs.** After deletion, run the registry's garbage collector to free disk space.
- **HTTPS assumed by default.** Plain HTTP registries must be specified with an explicit `http://` scheme in the `<REGISTRY>` argument.

---

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

---

## Contributing

Contributions are welcome! Please open an issue or submit a pull request on [GitHub](https://github.com/anthonyoteri/dredge).

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
