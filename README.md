# dredge

Dredge is a command line tool for working with the Docker Registry V2 API.

## Configuration

Dredge is configured using a simple TOML based configuration file format.
By default, it will look in the default configuration location for your
operating system i.e. on Linux this is `~/.config/dredge/dredge.toml`.

You may also override the configuration file at the command line with the
`-c` or `--config` argument.  The supplied argument must refer to an
existing file.

The format of the configuration file is simple, and contains only the
base URL for the registry server.

```toml
registry_url = "https://localhost:5000"
```

If Dredge is run without a configuration file present, one will be created
with the default configuration shown above.

## Usage

### Checking the API Version

Performs a simple API version check.

```sh
dredge check
```

### Fetch Repository List

Fetch a sorted list of available repository names from the Registry's
catalog.

```sh
dredge catalog
```
