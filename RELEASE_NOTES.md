# Dredge Release Notes

## Known Issues

* The delete command is currently not implemented and will return an error
  if called.
* Docker authentication is not currently supported, and attempts to query a
  registry which requires authentication will fail.

## Changelog
- v0.2.0

  - Support deleting an image tag by Anthony Oteri fbe43f0
  - Replace femme logger with simple_logger by Anthony Oteri 13ae092

- v0.1.0

  - Additional scripts for managing the release process by Anthony Oteri cfdefb2

