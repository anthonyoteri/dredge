/*
 * Copyright 2023 Anthony Oteri
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */

use std::io::Write;

use serde::Deserialize;
use serde::Serialize;
use url::Url;

use crate::api;
use crate::error::ApiError;

/// Deserialized body of a `/v2/_catalog` response page.
#[derive(Deserialize)]
struct CatalogResponse {
    repositories: Vec<String>,
}

/// Deserialized body of a `/v2/<name>/tags/list` response page.
#[derive(Deserialize)]
struct TagsResponse {
    tags: Vec<String>,
}

/// A single filesystem layer entry within a V1 image manifest.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FsLayer {
    blob_sum: String,
}

/// Deserialized body of a `/v2/<image>/manifests/<tag>` response, augmented
/// with the `digest` and `etag` values extracted from response headers.
#[derive(Debug, Serialize, Deserialize)]
struct ManifestResponse {
    name: String,
    tag: String,
    architecture: String,

    #[serde(rename = "fsLayers")]
    fslayers: Vec<FsLayer>,

    /// Content digest from the `docker-content-digest` response header.
    #[serde(skip_deserializing)]
    digest: String,

    /// `ETag` value from the response header (quotes stripped), or the digest
    /// when the `ETag` header is absent.
    #[serde(skip_deserializing)]
    etag: String,
}

/// Fetch all repository names from the registry catalog and write them to `buf`.
///
/// Queries `/v2/_catalog` via [`api::fetch_paginated`], collects all pages,
/// and writes one repository name per line to the provided writer.
///
/// # Arguments
///
/// * `buf` — Output sink (typically stdout or a test buffer).
/// * `registry_url` — Base URL of the Docker Registry.
///
/// # Errors
///
/// * [`ApiError::HttpError`] — the HTTP client could not be constructed, or a
///   request to the registry failed at the transport layer, or a response body
///   could not be decoded as JSON.
/// * [`ApiError::UrlParseError`] — the catalog URL could not be constructed.
/// * [`ApiError::ResponseHeaderParseError`] — a response header contains
///   non-UTF-8 bytes.
/// * [`ApiError::UnsupportedVersion`] — the registry version header has an
///   unexpected value.
/// * [`ApiError::UnexpectedResponse`] — a required response header is absent.
/// * [`ApiError::AuthorizationFailed`] — the registry requires authentication.
/// * [`ApiError::NotFound`] — the catalog endpoint does not exist.
/// * [`ApiError::MethodNotAllowed`] — the registry rejected the request method.
/// * [`ApiError::IOError`] — writing a repository name to `buf` failed.
pub async fn catalog_handler(buf: &mut dyn Write, registry_url: &Url) -> Result<(), ApiError> {
    log::trace!("catalog_handler(registry_url: {registry_url:?})");

    let client = api::build_client()?;
    let responses: Vec<CatalogResponse> =
        api::fetch_paginated(&client, registry_url, "v2/_catalog").await?;

    for repo in responses.iter().flat_map(|r| r.repositories.iter()) {
        writeln!(buf, "{repo}")?;
    }

    Ok(())
}

/// Fetch all tags for an image from the registry and write them to `buf`.
///
/// Queries `/v2/<name>/tags/list` via [`api::fetch_paginated`], collects all
/// pages, and writes one tag name per line to the provided writer.
///
/// # Arguments
///
/// * `buf` — Output sink (typically stdout or a test buffer).
/// * `registry_url` — Base URL of the Docker Registry.
/// * `name` — The repository name whose tags should be listed
///   (e.g. `"myorg/backend"`).
///
/// # Errors
///
/// * [`ApiError::HttpError`] — the HTTP client could not be constructed, or a
///   request to the registry failed at the transport layer, or a response body
///   could not be decoded as JSON.
/// * [`ApiError::UrlParseError`] — the tags URL could not be constructed.
/// * [`ApiError::ResponseHeaderParseError`] — a response header contains
///   non-UTF-8 bytes.
/// * [`ApiError::UnsupportedVersion`] — the registry version header has an
///   unexpected value.
/// * [`ApiError::UnexpectedResponse`] — a required response header is absent.
/// * [`ApiError::AuthorizationFailed`] — the registry requires authentication.
/// * [`ApiError::NotFound`] — the image does not exist in the registry.
/// * [`ApiError::MethodNotAllowed`] — the registry rejected the request method.
/// * [`ApiError::IOError`] — writing a tag name to `buf` failed.
pub async fn tags_handler(
    buf: &mut dyn Write,
    registry_url: &Url,
    name: &str,
) -> Result<(), ApiError> {
    log::trace!("tags_handler(registry_url: {registry_url:?}, name: {name})");

    let client = api::build_client()?;
    let responses: Vec<TagsResponse> =
        api::fetch_paginated(&client, registry_url, &format!("/v2/{name}/tags/list")).await?;

    for tag in responses.iter().flat_map(|r| r.tags.iter()) {
        writeln!(buf, "{tag}")?;
    }

    Ok(())
}

/// Fetch and display the manifest for a tagged image.
///
/// Queries `/v2/<image>/manifests/<tag>`, extracts the
/// `docker-content-digest` and `etag` response headers, deserializes the
/// manifest JSON body, and serializes the result as YAML to `buf`.
///
/// The output includes the image name, tag, target architecture, filesystem
/// layer digests, content digest, and `ETag`.
///
/// # Arguments
///
/// * `buf` — Output sink (typically stdout or a test buffer).
/// * `registry_url` — Base URL of the Docker Registry.
/// * `image` — The repository name (e.g. `"myorg/backend"`).
/// * `tag` — The tag to inspect (e.g. `"v2.0.0"`).  Pass `"latest"` when
///   no explicit tag was provided by the caller.
///
/// # Errors
///
/// * [`ApiError::HttpError`] — the HTTP client could not be constructed, or the
///   request failed at the transport layer, or the response body could not be
///   decoded as JSON.
/// * [`ApiError::UrlParseError`] — the manifest URL could not be constructed.
/// * [`ApiError::ResponseHeaderParseError`] — a response header contains
///   non-UTF-8 bytes.
/// * [`ApiError::UnexpectedResponse`] — the `docker-content-digest` header is
///   absent, or a required version header is missing.
/// * [`ApiError::UnsupportedVersion`] — the registry version header has an
///   unexpected value.
/// * [`ApiError::AuthorizationFailed`] — the registry requires authentication.
/// * [`ApiError::NotFound`] — the image or tag does not exist in the registry.
/// * [`ApiError::MethodNotAllowed`] — the registry rejected the request method.
/// * [`ApiError::SerializerError`] — the manifest could not be serialized to YAML.
/// * [`ApiError::IOError`] — writing the YAML output to `buf` failed.
#[allow(clippy::similar_names)]
pub async fn show_handler(
    buf: &mut dyn Write,
    registry_url: &Url,
    image: &str,
    tag: &str,
) -> Result<(), ApiError> {
    log::trace!("show_handler(registry_url: {registry_url:?}, image: {image}, tag: {tag})");
    let path = format!("/v2/{image}/manifests/{tag}");
    let url = registry_url.join(&path)?;

    let client = api::build_client()?;
    let resp = client.get(url).send().await?;
    api::parse_response_status(&resp)?;

    let headers = resp.headers();
    let digest = headers
        .get("docker-content-digest")
        .ok_or_else(|| ApiError::UnexpectedResponse("Missing docker-content-digest header".into()))?
        .to_str()?
        .to_owned();

    // Docker Registry API ETags are quoted strings per RFC 7232, e.g.
    // `"sha256:abc123"`.  Strip surrounding double-quotes when present; fall
    // back to the digest when the header is absent.
    let etag = match headers.get("etag") {
        Some(v) => {
            let raw = v.to_str()?;
            raw.strip_prefix('"')
                .and_then(|s| s.strip_suffix('"'))
                .unwrap_or(raw)
                .to_owned()
        }
        None => digest.clone(),
    };

    let mut body: ManifestResponse = resp.json().await?;
    body.digest = digest;
    body.etag = etag;

    serde_norway::to_writer(buf, &body)?;
    Ok(())
}

/// Delete the manifest for a tagged image from the registry.
///
/// Resolves `tag` to its content digest by sending a `HEAD` request to
/// `/v2/<image>/manifests/<tag>`, then deletes the manifest by digest via
/// `DELETE /v2/<image>/manifests/<digest>`.
///
/// The registry must have storage deletion enabled.  Set the environment
/// variable `REGISTRY_STORAGE_DELETE_ENABLED=true` on the registry container.
/// If deletion is not enabled the registry returns `405 Method Not Allowed`
/// and this function returns [`ApiError::MethodNotAllowed`].
///
/// Only the manifest is removed.  Unreferenced layer blobs remain on disk
/// until the registry garbage collector is run separately.
///
/// # Arguments
///
/// * `_buf` — Unused output sink (reserved for future use).
/// * `registry_url` — Base URL of the Docker Registry.
/// * `image` — The repository name (e.g. `"myorg/backend"`).
/// * `tag` — The tag to delete (e.g. `"v1.0.0"`).
///
/// # Errors
///
/// * [`ApiError::HttpError`] — the HTTP client could not be constructed, or a
///   request failed at the transport layer.
/// * [`ApiError::UrlParseError`] — a manifest URL could not be constructed.
/// * [`ApiError::ResponseHeaderParseError`] — a response header contains
///   non-UTF-8 bytes.
/// * [`ApiError::UnexpectedResponse`] — the `docker-content-digest` header is
///   absent from the `HEAD` response, or a required version header is missing.
/// * [`ApiError::UnsupportedVersion`] — the registry version header has an
///   unexpected value.
/// * [`ApiError::AuthorizationFailed`] — the registry requires authentication.
/// * [`ApiError::NotFound`] — the image or tag does not exist in the registry.
/// * [`ApiError::MethodNotAllowed`] — the registry does not permit deletion;
///   ensure `REGISTRY_STORAGE_DELETE_ENABLED=true` is set on the registry.
#[allow(clippy::unused_async)]
pub async fn delete_handler(
    _buf: &mut dyn Write,
    registry_url: &Url,
    image: &str,
    tag: &str,
) -> Result<(), ApiError> {
    log::trace!("delete_handler(registry_url: {registry_url:?}, image: {image}, tag: {tag})");

    let client = api::build_client()?;
    let url = registry_url.join(&format!("/v2/{image}/manifests/{tag}"))?;
    let digest = api::get_digest(&client, &url).await?;

    log::debug!("Deleting digest {digest}");
    let url = registry_url.join(&format!("/v2/{image}/manifests/{digest}"))?;
    let resp = client.delete(url).send().await?;
    api::parse_response_status(&resp)?;

    Ok(())
}

/// Verify that the registry endpoint implements Docker Distribution API v2.
///
/// Sends a `GET` request to `/v2` and validates the response with
/// [`api::parse_response_status`].  On success writes `"Ok\n"` to `buf`.
///
/// # Arguments
///
/// * `buf` — Output sink (typically stdout or a test buffer).
/// * `registry_url` — Base URL of the Docker Registry.
///
/// # Errors
///
/// * [`ApiError::HttpError`] — the HTTP client could not be constructed, or the
///   request failed at the transport layer.
/// * [`ApiError::UrlParseError`] — the `/v2` URL could not be constructed.
/// * [`ApiError::ResponseHeaderParseError`] — the version header contains
///   non-UTF-8 bytes.
/// * [`ApiError::UnexpectedResponse`] — the `Docker-Distribution-API-Version`
///   header is absent from the response.
/// * [`ApiError::UnsupportedVersion`] — the version header has a value other
///   than `"registry/2.0"`.
/// * [`ApiError::AuthorizationFailed`] — the registry requires authentication.
/// * [`ApiError::NotFound`] — the `/v2` endpoint does not exist.
/// * [`ApiError::MethodNotAllowed`] — the registry rejected the request method.
/// * [`ApiError::IOError`] — writing `"Ok\n"` to `buf` failed.
pub async fn check_handler(buf: &mut dyn Write, registry_url: &Url) -> Result<(), ApiError> {
    log::trace!("check_handler(registry_url: {registry_url:?})");

    let path = "/v2";
    let url = registry_url.join(path)?;

    let client = api::build_client()?;
    let resp = client.get(url).send().await?;
    api::parse_response_status(&resp)?;
    writeln!(buf, "Ok")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use indoc::indoc;
    use url::Url;

    use crate::error;

    use super::*;

    /// Validate the happy path for the catalog handler.
    ///
    /// This test spins up a mock server, and makes a request to the catalog
    /// endpoint.  It checks that the handler both called the request the
    /// expected number of times, and did not return an error.
    #[tokio::test]
    async fn test_catalog_handler() {
        let mut server = mockito::Server::new_async().await;
        let path = "/v2/_catalog";

        let registry_url = Url::parse(&server.url()).expect("Failed to parse registry URL");
        let mock_response = server
            .mock("GET", path)
            .with_status(http::status::StatusCode::OK.as_u16().into())
            .with_header(http::header::CONTENT_TYPE.as_str(), "application/json")
            .with_header("Docker-Distribution-API-Version", "registry/2.0")
            .with_body(r#"{"repositories": ["image1", "image2", "image3"]}"#)
            .create();

        let mut buf: Vec<u8> = Vec::new();
        let result = catalog_handler(&mut buf, &registry_url).await;
        assert!(result.is_ok(), "{:?}", result.unwrap_err());
        assert_eq!(String::from_utf8(buf).unwrap(), *"image1\nimage2\nimage3\n");

        mock_response.assert();
    }

    /// Validate the pagination of the catalog handler.
    ///
    /// This test spins up a mock server, and makes a request to the catalog
    /// endpoint.  The response includes a pagination link, which the handler
    /// should follow, resulting in the combined list.  It checks that the
    /// handler both called the request the expected number of times, and did
    /// not return an error.
    #[tokio::test]
    async fn test_catalog_handler_with_pagination() {
        let mut server = mockito::Server::new_async().await;
        let path = "/v2/_catalog";
        let path2 = "/v2/_catalog?n=2,last=image2";

        let registry_url = Url::parse(&server.url()).expect("Failed to parse registry URL");
        let mock_response = server
            .mock("GET", path)
            .with_status(http::status::StatusCode::OK.as_u16().into())
            .with_header(http::header::CONTENT_TYPE.as_str(), "application/json")
            .with_header("Docker-Distribution-API-Version", "registry/2.0")
            .with_header(
                http::header::LINK.as_str(),
                &format!(r"<{path2}>; rel=next"),
            )
            .with_body(r#"{"repositories": ["image1", "image2"]}"#)
            .create();

        let mock_response2 = server
            .mock("GET", path2)
            .with_status(http::status::StatusCode::OK.as_u16().into())
            .with_header(http::header::CONTENT_TYPE.as_str(), "application/json")
            .with_header("Docker-Distribution-API-Version", "registry/2.0")
            .with_body(r#"{"repositories": ["image3"]}"#)
            .create();

        let mut buf: Vec<u8> = Vec::new();
        let result = catalog_handler(&mut buf, &registry_url).await;
        assert!(result.is_ok(), "{:?}", result.unwrap_err());
        assert_eq!(String::from_utf8(buf).unwrap(), *"image1\nimage2\nimage3\n");

        mock_response.assert();
        mock_response2.assert();
    }

    /// Validate the happy path for the tags handler.
    ///
    /// This test spins up a mock server, and makes a request to the tags
    /// endpoint.  It checks that the handler both called the request the
    /// expected number of times, and did not return an error.
    #[tokio::test]
    async fn test_tags_handler() {
        let mut server = mockito::Server::new_async().await;
        let path = "/v2/some_image/tags/list";

        // Mock the HTTP response for the Docker Registry API
        let registry_url = Url::parse(&server.url()).expect("Failed to parse registry URL");
        let mock_response = server
            .mock("GET", path)
            .with_status(http::status::StatusCode::OK.as_u16().into())
            .with_header(http::header::CONTENT_TYPE.as_str(), "application/json")
            .with_header("Docker-Distribution-API-Version", "registry/2.0")
            .with_body(r#"{"tags": ["tag1", "tag2", "tag3"]}"#)
            .create();

        let mut buf: Vec<u8> = Vec::new();
        let result = tags_handler(&mut buf, &registry_url, "some_image").await;
        assert!(result.is_ok(), "{:?}", result.unwrap_err());
        assert_eq!(String::from_utf8(buf).unwrap(), *"tag1\ntag2\ntag3\n");

        mock_response.assert();
    }

    /// Validate the pagination of the tags handler.
    ///
    /// This test spins up a mock server, and makes a request to the tags
    /// endpoint.  The response includes a pagination link, which the handler
    /// should follow, resulting in the combined list.  It checks that the
    /// handler both called the request the expected number of times, and did
    /// not return an error.
    #[tokio::test]
    async fn test_tags_handler_with_pagination() {
        let mut server = mockito::Server::new_async().await;
        let path = "/v2/some_image/tags/list";
        let path2 = "/v2/some_image/tags/list?n=2,last=tag2";

        // Mock the HTTP response for the Docker Registry API
        let registry_url = Url::parse(&server.url()).expect("Failed to parse registry URL");
        let mock_response = server
            .mock("GET", path)
            .with_status(http::status::StatusCode::OK.as_u16().into())
            .with_header(http::header::CONTENT_TYPE.as_str(), "application/json")
            .with_header("Docker-Distribution-API-Version", "registry/2.0")
            .with_header(
                http::header::LINK.as_str(),
                &format!(r"<{path2}>; rel=next"),
            )
            .with_body(r#"{"tags": ["tag1", "tag2"]}"#)
            .create();

        let mock_response2 = server
            .mock("GET", path2)
            .with_status(http::status::StatusCode::OK.as_u16().into())
            .with_header(http::header::CONTENT_TYPE.as_str(), "application/json")
            .with_header("Docker-Distribution-API-Version", "registry/2.0")
            .with_body(r#"{"tags": ["tag3"]}"#)
            .create();

        let mut buf: Vec<u8> = Vec::new();
        let result = tags_handler(&mut buf, &registry_url, "some_image").await;
        assert!(result.is_ok(), "{:?}", result.unwrap_err());
        assert_eq!(String::from_utf8(buf).unwrap(), *"tag1\ntag2\ntag3\n");

        mock_response.assert();
        mock_response2.assert();
    }

    /// Validate the happy path for the check handler.
    ///
    /// This test spins up a mock server, and makes a request to the check
    /// endpoint.  It checks that the handler both called the request the
    /// expected number of times, and did not return an error.
    #[tokio::test]
    async fn test_check_handler() {
        let mut server = mockito::Server::new_async().await;
        let path = "/v2";

        // Mock the HTTP response for the Docker Registry API
        let registry_url = Url::parse(&server.url()).expect("Failed to parse registry URL");
        let mock_response = server
            .mock("GET", path)
            .with_status(http::status::StatusCode::OK.as_u16().into())
            .with_header(http::header::CONTENT_TYPE.as_str(), "application/json")
            .with_header("Docker-Distribution-API-Version", "registry/2.0")
            .create();

        let mut buf: Vec<u8> = Vec::new();
        let result = check_handler(&mut buf, &registry_url).await;
        assert!(result.is_ok(), "{:?}", result.unwrap_err());
        assert_eq!(String::from_utf8(buf).unwrap(), *"Ok\n");

        mock_response.assert();
    }

    /// Validate the check handler when the API version header is missing.
    ///
    /// This validates that if the "Docker-Distribution-API-Version" header
    /// is missing in the response, the appropriate error is returned.
    #[tokio::test]
    async fn test_check_handler_missing_api_version() -> Result<(), Box<dyn Error>> {
        let mut server = mockito::Server::new_async().await;
        let path = "/v2";

        // Mock the HTTP response for the Docker Registry API
        let registry_url = Url::parse(&server.url()).expect("Failed to parse registry URL");
        let mock_response = server
            .mock("GET", path)
            .with_status(http::status::StatusCode::OK.as_u16().into())
            .with_header(http::header::CONTENT_TYPE.as_str(), "application/json")
            .create();

        let mut buf: Vec<u8> = Vec::new();
        let result = check_handler(&mut buf, &registry_url).await;

        // Ensure that we got the correct error type.
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            error::ApiError::UnexpectedResponse(_) => Ok(()),
            e => Err(e),
        }?;

        mock_response.assert();
        Ok(())
    }

    /// Validate the check handler when the API version header has an unexpected value.
    ///
    /// This validates that if the "Docker-Distribution-API-Version" header
    /// is present in the response but contains an unexpected value, the
    /// appropriate error is returned.
    #[tokio::test]
    async fn test_check_handler_invalid_api_version() -> Result<(), Box<dyn Error>> {
        let mut server = mockito::Server::new_async().await;
        let path = "/v2";

        // Mock the HTTP response for the Docker Registry API
        let registry_url = Url::parse(&server.url()).expect("Failed to parse registry URL");
        let mock_response = server
            .mock("GET", path)
            .with_status(http::status::StatusCode::OK.as_u16().into())
            .with_header(http::header::CONTENT_TYPE.as_str(), "application/json")
            .with_header("Docker-Distribution-API-Version", "registry/1.0")
            .create();

        let mut buf: Vec<u8> = Vec::new();
        let result = check_handler(&mut buf, &registry_url).await;

        // Ensure that we got the correct error type.
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            error::ApiError::UnsupportedVersion(_) => Ok(()),
            e => Err(e),
        }?;

        mock_response.assert();
        Ok(())
    }

    /// Validate the happy path for the show handler.
    ///
    /// This test spins up a mock server, and makes a request to the image
    /// manifests endpoint.  It checks that the handler both called the request
    /// the expected number of times, and did not return an error.
    #[tokio::test]
    async fn test_show_handler() {
        let mut server = mockito::Server::new_async().await;
        let path = "/v2/foo/manifests/latest";

        let response_body = r#"
        {
               "schemaVersion": 1,
               "name": "foo",
               "tag": "latest",
               "architecture": "amd64",
               "fsLayers": [
                  {
                     "blobSum": "sha256:a3ed95caeb02ffe68cdd9fd84406680ae93d633cb16422d00e8a7c22955b46d4"
                  },
                  {
                     "blobSum": "sha256:7d97e254a0461b0a30b3f443f1daa0d620a3cc6ff4e2714cc1cfd96ace5b7a7e"
                  }
               ],
               "history": [
                  {
                     "v1Compatibility": "{\"id\":\"7fe38ce3fe63caeaacf6be64933d0d55adc5c5f48762b20ec6129d1a41691a84\",\"parent\":\"8ca907037d044ff942e9c95562b786f1913d3b05a4bda16ad3ed3e7ee67e8c76\",\"created\":\"2023-09-07T00:21:13.838729514Z\",\"container_config\":{\"Cmd\":[\"/bin/sh -c #(nop)  CMD [\\\"bash\\\"]\"]},\"throwaway\":true}"
                  },
                  {
                     "v1Compatibility": "{\"id\":\"8ca907037d044ff942e9c95562b786f1913d3b05a4bda16ad3ed3e7ee67e8c76\",\"created\":\"2023-09-07T00:21:13.444807009Z\",\"container_config\":{\"Cmd\":[\"/bin/sh -c #(nop) ADD file:cb5fcc80c057b356a31492a20c6e3a75b70ed70a663506c8e97ad730ae32a02d in / \"]}}"
                  }
               ],
               "signatures": [
                  {
                     "header": {
                        "jwk": {
                           "crv": "P-256",
                           "kid": "7ZLW:DJCO:GYG4:DCZD:TRO6:QW3Y:Q7Q3:PTXB:JDQX:4DLY:NB2B:4GJJ",
                           "kty": "EC",
                           "x": "LXquBoF1_XI3fawa-7UW9Y1Le7j7FiDGS3KB_4gF5hY",
                           "y": "UT5SniKpELMqL-j9YwL2fZLUHmRIFwori9rUBG18b_k"
                        },
                        "alg": "ES256"
                     },
                     "signature": "5_paRRhUCmwkAZJrjBfbvOJ341atEjUQuhG7i4kITyG3e_U2yuDqs9X7bHHMtmUTbChSp59NHi124uauAjoxIg",
                     "protected": "eyJmb3JtYXRMZW5ndGgiOjI3MDIsImZvcm1hdFRhaWwiOiJDbjAiLCJ0aW1lIjoiMjAyMy0wOS0yN1QxMzoyMTo1MloifQ"
                  }
               ]
            }
        "#;
        // Mock the HTTP response for the Docker Registry API
        let registry_url = Url::parse(&server.url()).expect("Failed to parse registry URL");
        let mock_response = server
            .mock("GET", path)
            .with_status(http::status::StatusCode::OK.as_u16().into())
            .with_header(http::header::CONTENT_TYPE.as_str(), "application/json")
            .with_header("Docker-Distribution-API-Version", "registry/2.0")
            .with_header(
                "docker-content-digest",
                "sha256:0259571889ac87efbfca5b79a0abe9baf626d058ec5f9a5744bace2229d9ed50",
            )
            .with_header(
                "etag",
                "sha256:0259571889ac87efbfca5b79a0abe9baf626d058ec5f9a5744bace2229d9ed50",
            )
            .with_body(response_body)
            .create();

        let mut buf: Vec<u8> = Vec::new();
        let result = show_handler(&mut buf, &registry_url, "foo", "latest").await;

        let expected_body = indoc! {"
        name: foo
        tag: latest
        architecture: amd64
        fsLayers:
        - blobSum: sha256:a3ed95caeb02ffe68cdd9fd84406680ae93d633cb16422d00e8a7c22955b46d4
        - blobSum: sha256:7d97e254a0461b0a30b3f443f1daa0d620a3cc6ff4e2714cc1cfd96ace5b7a7e
        digest: sha256:0259571889ac87efbfca5b79a0abe9baf626d058ec5f9a5744bace2229d9ed50
        etag: sha256:0259571889ac87efbfca5b79a0abe9baf626d058ec5f9a5744bace2229d9ed50\n"
        };

        assert!(result.is_ok(), "{:?}", result.unwrap_err());
        assert_eq!(String::from_utf8(buf).unwrap(), *expected_body);

        mock_response.assert();
    }
}
