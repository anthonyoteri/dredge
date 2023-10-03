# dredge

Dredge is a command line tool for working with the Docker Registry V2 API.

## Usage

```shell
Dredge is a command line tool for working with the Docker Registry V2 API.

Usage: dredge [OPTIONS] <REGISTRY> <COMMAND>

Commands:
  catalog  Fetch the list of available repositories from the catalog
  tags     Fetch the list of tags for a given image
  show     Show detailed information about a particular image
  delete   Delete a tagged image from the registry
  check    Perform a simple API Version check towards the configured registry endpoint
  help     Print this message or the help of the given subcommand(s)

Arguments:
  <REGISTRY>
          The host or host:port or full base URL of the Docker Registry

Options:
      --log-level[=<LEVEL>]
          [default: info]
          [possible values: trace, debug, info, warn, error, off]

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

### Checking the API Version

Perform a simple API Version check towards the registry endpoint

```sh
Usage: dredge <REGISTRY> check

Options:
  -h, --help  Print help
```

### Fetch Repository List

Fetch the list of available repositories from the catalog

```sh
Usage: dredge <REGISTRY> catalog

Options:
  -h, --help  Print help
```

### Listing tags for an image

Fetch the list of tags for a given image

```shell
Usage: dredge <REGISTRY> tags <NAME>

Arguments:
  <NAME>  

Options:
  -h, --help  Print help
```

### Viewing details of a tagged image

Show detailed information about a particular image

```shell
Usage: dredge <REGISTRY> show <IMAGE> [TAG]

Arguments:
<IMAGE>  
[TAG]

Options:
-h, --help  Print help
```

### Deleteing a tagged image

Delete a tagged image from the registry

Note! This requires that the registry has storage delete rights enabled. For
example, when creating the registry, setting the environment variable
`REGISTRY_STORAGE_DELETE_ENABLED=true` to enable that feature. If that is not
enabled, a `MethodNotAllowed` error will be returned.

Note! This will only remove the tag from the registry, it will not remove
orphaned digests. For that, the garbage collector on the registry service must
be run separately.

```shell
Usage: dredge <REGISTRY> delete <IMAGE> <TAG>

Arguments:
<IMAGE>  
<TAG>

Options:
-h, --help  Print help
```

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.

