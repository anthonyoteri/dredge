# Dredge Release Notes

## Legal

As of version 1.1.0, this software license has been changed from Apache-2.0
to a dual-licensed Apache-2.0 OR MIT license.

## Known Issues

* Docker authentication is not currently supported, and attempts to query a
  registry which requires authentication will fail.

## Changelog
- v1.1.0

  - Change License by Anthony Oteri 0e4219b

- v1.0.0

  - Rename project to dredge-tool by Anthony Oteri b60d433
  - Replace async_std::test with tokio::test by Anthony Oteri 42f8f46
  - Replace async-std with tokio by Anthony Oteri 80d1acf
  - Update known issues in release notes by Anthony Oteri 12dd298

- v0.2.0

  - Support deleting an image tag by Anthony Oteri fbe43f0
  - Replace femme logger with simple_logger by Anthony Oteri 13ae092

- v0.1.0

  - Additional scripts for managing the release process by Anthony Oteri cfdefb2

