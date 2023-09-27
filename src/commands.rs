/*
 *    Copyright 2023 Anthony Oteri
 *
 *    Licensed under the Apache License, Version 2.0 (the "License");
 *    you may not use this file except in compliance with the License.
 *    You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 *    Unless required by applicable law or agreed to in writing, software
 *    distributed under the License is distributed on an "AS IS" BASIS,
 *    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *    See the License for the specific language governing permissions and
 *    limitations under the License.
 */

use std::io::Write;

use serde::Deserialize;
use serde::Serialize;
use url::Url;

use crate::api;
use crate::error::ApiError;

/// Handler for the `Catalog` endpoint
///
/// Fetch the list of repository names from the Docker Registry API, and
/// simply print the resulting names to stdout.
///
/// # Errors:
///
/// Returns an `ApiError` if there is a problem fetching or parsing the
/// responses from the Docker Registry API.  
pub async fn catalog_handler(buf: &mut dyn Write, registry_url: &Url) -> Result<(), ApiError> {
    #[derive(Deserialize)]
    struct Response {
        repositories: Vec<String>,
    }

    log::trace!("catalog_handler(registry_url: {registry_url:?})");
    let path = "v2/_catalog";

    let responses: Vec<Response> = api::fetch_paginated(registry_url, path).await?;
    let repository_list: Vec<&str> = responses
        .iter()
        .flat_map(|r| r.repositories.iter().map(String::as_str))
        .collect();

    for repository in repository_list {
        writeln!(buf, "{repository}")?;
    }

    Ok(())
}

/// Handler for the `Tags` endpoint
///
/// Fetch the list of tags names for a given image from the Docker Registry API, and
/// simply print the resulting names to stdout.
///
/// # Errors:
///
/// Returns an `ApiError` if there is a problem fetching or parsing the
/// responses from the Docker Registry API.  
pub async fn tags_handler(
    buf: &mut dyn Write,
    registry_url: &Url,
    name: &str,
) -> Result<(), ApiError> {
    #[derive(Deserialize)]
    struct Response {
        tags: Vec<String>,
    }

    log::trace!("tags_handler(registry_url: {registry_url:?}, name: {name})");
    let path = format!("/v2/{name}/tags/list");

    let responses: Vec<Response> = api::fetch_paginated(registry_url, &path).await?;
    let tag_list: Vec<&str> = responses
        .iter()
        .flat_map(|r| r.tags.iter().map(String::as_str))
        .collect();

    for tag in tag_list {
        writeln!(buf, "{tag}")?;
    }

    Ok(())
}

/// Handler function for showing manifest details
///
/// # Errors:
///
/// Returns an `ApiError` if there is a problem fetching the manifest or if there
/// is a problem parsing the response from the Docker Registry API.
#[allow(clippy::similar_names)]
pub async fn show_handler(
    buf: &mut dyn Write,
    registry_url: &Url,
    image: &str,
    tag: &str,
) -> Result<(), ApiError> {
    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct FsLayer {
        blob_sum: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct Response {
        name: String,
        tag: String,
        architecture: String,

        #[serde(rename = "fsLayers")]
        fslayers: Vec<FsLayer>,

        #[serde(skip_deserializing)]
        digest: String,

        #[serde(skip_deserializing)]
        etag: String,
    }
    log::trace!("show_handler(registry_url: {registry_url:?}, image: {image}, tag: {tag})");
    let path = format!("/v2/{image}/manifests/{tag}");
    let url = registry_url.join(&path)?;

    let resp = reqwest::get(url).await?;
    let headers = resp.headers();
    let digest: String = String::from(
        headers
            .get("docker-content-digest")
            .ok_or(ApiError::UnexpectedResponse(String::from(
                "Missing docker-content-digest header",
            )))?
            .to_str()?,
    );

    let etag: String = String::from(
        headers
            .get("etag")
            .ok_or(ApiError::UnexpectedResponse(String::from(
                "Missing etag header",
            )))?
            .to_str()?
            .strip_prefix("'\"")
            .and_then(|s| s.strip_suffix("\"'"))
            .unwrap_or(&digest),
    );

    let mut body: Response = resp.json().await?;
    body.digest = digest;
    body.etag = etag;

    serde_yaml::to_writer(buf, &body)?;
    Ok(())
}

/// Handler function for deleting a manifest for a given tagged image.
///
/// # Errors:
///
/// Returns and `ApiError` if there is a problem converting the given tag to a
/// manifest digest, or if there is a problem deleting the manifest from the
/// Docker Registry API.
#[allow(clippy::unused_async)]
pub async fn delete_handler(
    _buf: &mut dyn Write,
    registry_url: &Url,
    image: &str,
    tag: &str,
) -> Result<(), ApiError> {
    log::trace!("delete_handler(registry_url: {registry_url:?}, image: {image}, tag: {tag})");
    todo!()
}

// Path to the Docker Registry APIs "api version check" endpoint.

/// Handler for the API Version Check.
///
/// # Errors:
///
/// Returns an `ApiError` if there is a problem communicating with the
/// endpoint or if the required version is not supported.
pub async fn check_handler(buf: &mut dyn Write, registry_url: &Url) -> Result<(), ApiError> {
    log::trace!("check_handler(registry_url: {registry_url:?})");

    let path = "/v2";
    let url = registry_url.join(path)?;

    let response = reqwest::get(url).await?;
    api::parse_response_status(&response)?;
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
    #[async_std::test]
    async fn test_catalog_handler() {
        let mut server = mockito::Server::new_async().await;
        let path = "/v2/_catalog";

        let registry_url = Url::parse(&server.url()).expect("Failed to parse registry URL");
        let mock_response = server
            .mock("GET", path)
            .with_status(http::status::StatusCode::OK.as_u16().into())
            .with_header(http::header::CONTENT_TYPE.as_str(), "application/json")
            .with_body(r#"{"repositories": ["image1", "image2", "image3"]}"#)
            .create();

        let mut buf: Vec<u8> = Vec::new();
        let result = catalog_handler(&mut buf, &registry_url).await;
        assert!(result.is_ok());
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
    #[async_std::test]
    async fn test_catalog_handler_with_pagination() {
        let mut server = mockito::Server::new_async().await;
        let path = "/v2/_catalog";
        let path2 = "/v2/_catalog?n=2,last=image2";

        let registry_url = Url::parse(&server.url()).expect("Failed to parse registry URL");
        let mock_response = server
            .mock("GET", path)
            .with_status(http::status::StatusCode::OK.as_u16().into())
            .with_header(http::header::CONTENT_TYPE.as_str(), "application/json")
            .with_header(
                http::header::LINK.as_str(),
                &format!(r#"<{path2}>; rel=next"#),
            )
            .with_body(r#"{"repositories": ["image1", "image2"]}"#)
            .create();

        let mock_response2 = server
            .mock("GET", path2)
            .with_status(http::status::StatusCode::OK.as_u16().into())
            .with_header(http::header::CONTENT_TYPE.as_str(), "application/json")
            .with_body(r#"{"repositories": ["image3"]}"#)
            .create();

        let mut buf: Vec<u8> = Vec::new();
        let result = catalog_handler(&mut buf, &registry_url).await;
        assert!(result.is_ok());
        assert_eq!(String::from_utf8(buf).unwrap(), *"image1\nimage2\nimage3\n");

        mock_response.assert();
        mock_response2.assert();
    }

    /// Validate the happy path for the tags handler.
    ///
    /// This test spins up a mock server, and makes a request to the tags
    /// endpoint.  It checks that the handler both called the request the
    /// expected number of times, and did not return an error.
    #[async_std::test]
    async fn test_tags_handler() {
        let mut server = mockito::Server::new_async().await;
        let path = "/v2/some_image/tags/list";

        // Mock the HTTP response for the Docker Registry API
        let registry_url = Url::parse(&server.url()).expect("Failed to parse registry URL");
        let mock_response = server
            .mock("GET", path)
            .with_status(http::status::StatusCode::OK.as_u16().into())
            .with_header(http::header::CONTENT_TYPE.as_str(), "application/json")
            .with_body(r#"{"tags": ["tag1", "tag2", "tag3"]}"#)
            .create();

        let mut buf: Vec<u8> = Vec::new();
        let result = tags_handler(&mut buf, &registry_url, "some_image").await;
        assert!(result.is_ok());
        assert_eq!(String::from_utf8(buf).unwrap(), *"tag1\ntag2\ntag3\n");

        mock_response.assert();
    }

    /// Validate the pagination of the catalog handler.
    ///
    /// This test spins up a mock server, and makes a request to the catalog
    /// endpoint.  The response includes a pagination link, which the handler
    /// should follow, resulting in the combined list.  It checks that the
    /// handler both called the request the expected number of times, and did
    /// not return an error.
    #[async_std::test]
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
            .with_header(
                http::header::LINK.as_str(),
                &format!(r#"<{path2}>; rel=next"#),
            )
            .with_body(r#"{"tags": ["tag1", "tag2"]}"#)
            .create();

        let mock_response2 = server
            .mock("GET", path2)
            .with_status(http::status::StatusCode::OK.as_u16().into())
            .with_header(http::header::CONTENT_TYPE.as_str(), "application/json")
            .with_body(r#"{"tags": ["tag3"]}"#)
            .create();

        let mut buf: Vec<u8> = Vec::new();
        let result = tags_handler(&mut buf, &registry_url, "some_image").await;
        assert!(result.is_ok());
        assert_eq!(String::from_utf8(buf).unwrap(), *"tag1\ntag2\ntag3\n");

        mock_response.assert();
        mock_response2.assert();
    }

    /// Validate the happy path for the check handler.
    ///
    /// This test spins up a mock server, and makes a request to the check
    /// endpoint.  It checks that the handler both called the request the
    /// expected number of times, and did not return an error.
    #[async_std::test]
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
        assert!(result.is_ok());
        assert_eq!(String::from_utf8(buf).unwrap(), *"Ok\n");

        mock_response.assert();
    }

    /// Validate the the check handler on invalid API version
    ///
    /// This validates that if the "Docker-Distribution-API-Version" header
    /// is missing in the response, the appropriate error is returned.
    #[async_std::test]
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

    /// Validate the the check handler on invalid API version
    ///
    /// This validates that if the "Docker-Distribution-API-Version" header
    /// is present in the response but contains an unexpected value, the
    /// appropriate error is returned.
    #[async_std::test]
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
    #[async_std::test]
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

        assert!(result.is_ok());
        assert_eq!(String::from_utf8(buf).unwrap(), *expected_body);

        mock_response.assert();
    }
}
