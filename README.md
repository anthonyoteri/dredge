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

```shell
Usage: dredge <REGISTRY> delete <IMAGE> <TAG>

Arguments:
<IMAGE>  
<TAG>

Options:
-h, --help  Print help
```