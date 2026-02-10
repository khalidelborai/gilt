//! HTTP/reqwest integration module -- pre-built progress for HTTP requests.
//!
//! Provides an extension trait for [`reqwest::RequestBuilder`] that adds progress
//! tracking to HTTP requests, with curl-style progress display showing transfer
//! speed, downloaded size, total size, ETA, and a progress bar.
//!
//! # Features
//!
//! This module requires the `http` feature to be enabled:
//!
//! ```toml
//! [dependencies]
//! gilt = { version = "0.8", features = ["http"] }
//! ```
//!
//! # Examples
//!
//! ## Simple download with progress
//!
//! ```rust,no_run
//! use gilt::http::RequestBuilderProgress;
//!
//! # async fn example() -> reqwest::Result<()> {
//! let response = reqwest::Client::new()
//!     .get("https://api.example.com/data.json")
//!     .with_progress("Downloading")
//!     .send()
//!     .await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Download to file
//!
//! ```rust,no_run
//! use std::path::Path;
//!
//! # async fn example() -> reqwest::Result<()> {
//! let bytes = gilt::http::download_with_progress(
//!     "https://example.com/file.zip",
//!     Path::new("file.zip"),
//!     "Downloading file"
//! ).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Read JSON response with progress
//!
//! ```rust,no_run
//! use serde::Deserialize;
//! use gilt::http::RequestBuilderProgress;
//!
//! #[derive(Deserialize)]
//! struct Data {
//!     name: String,
//! }
//!
//! # async fn example() -> reqwest::Result<()> {
//! let data: Data = reqwest::Client::new()
//!     .get("https://api.example.com/data.json")
//!     .with_progress("Fetching data")
//!     .send()
//!     .await?
//!     .json()
//!     .await?;
//! # Ok(())
//! # }
//! ```

use std::fmt;
use std::io;
use std::path::Path;

use bytes::Bytes;
use reqwest::{Client, RequestBuilder, Response, StatusCode};

use crate::progress::{
    BarColumn, DownloadColumn, Progress, ProgressColumn, TaskId, TextColumn, TimeRemainingColumn,
    TransferSpeedColumn,
};

// Re-export reqwest types for convenience
pub use reqwest::{Error, Result};

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

/// Error type for text() method.
#[derive(Debug)]
pub enum TextError {
    /// A reqwest error occurred.
    Reqwest(reqwest::Error),
    /// The response body was not valid UTF-8.
    Utf8(std::string::FromUtf8Error),
}

impl fmt::Display for TextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TextError::Reqwest(e) => write!(f, "request error: {}", e),
            TextError::Utf8(e) => write!(f, "UTF-8 error: {}", e),
        }
    }
}

impl std::error::Error for TextError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            TextError::Reqwest(e) => Some(e),
            TextError::Utf8(e) => Some(e),
        }
    }
}

impl From<reqwest::Error> for TextError {
    fn from(e: reqwest::Error) -> Self {
        TextError::Reqwest(e)
    }
}

/// Error type for json() method.
#[derive(Debug)]
pub enum JsonError {
    /// A reqwest error occurred.
    Reqwest(reqwest::Error),
    /// JSON parsing failed.
    Json(serde_json::Error),
}

impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JsonError::Reqwest(e) => write!(f, "request error: {}", e),
            JsonError::Json(e) => write!(f, "JSON error: {}", e),
        }
    }
}

impl std::error::Error for JsonError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            JsonError::Reqwest(e) => Some(e),
            JsonError::Json(e) => Some(e),
        }
    }
}

impl From<reqwest::Error> for JsonError {
    fn from(e: reqwest::Error) -> Self {
        JsonError::Reqwest(e)
    }
}

/// Error type for download operations.
#[derive(Debug)]
pub enum DownloadError {
    /// An HTTP error occurred.
    Http(reqwest::Error),
    /// An I/O error occurred.
    Io(io::Error),
}

impl fmt::Display for DownloadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DownloadError::Http(e) => write!(f, "HTTP error: {}", e),
            DownloadError::Io(e) => write!(f, "I/O error: {}", e),
        }
    }
}

impl std::error::Error for DownloadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DownloadError::Http(e) => Some(e),
            DownloadError::Io(e) => Some(e),
        }
    }
}

impl From<reqwest::Error> for DownloadError {
    fn from(e: reqwest::Error) -> Self {
        DownloadError::Http(e)
    }
}

impl From<io::Error> for DownloadError {
    fn from(e: io::Error) -> Self {
        DownloadError::Io(e)
    }
}

// ---------------------------------------------------------------------------
// RequestBuilderProgress trait
// ---------------------------------------------------------------------------

/// Extension trait for [`reqwest::RequestBuilder`] that adds progress tracking.
///
/// This trait provides the [`with_progress`](RequestBuilderProgress::with_progress)
/// method to wrap a request builder with progress tracking.
///
/// # Examples
///
/// ```rust,no_run
/// use gilt::http::RequestBuilderProgress;
///
/// # async fn example() -> reqwest::Result<()> {
/// let response = reqwest::Client::new()
///     .get("https://example.com/file.zip")
///     .with_progress("Downloading file")
///     .send()
///     .await?;
/// # Ok(())
/// # }
/// ```
pub trait RequestBuilderProgress: Sized {
    /// Add progress tracking to this request.
    ///
    /// Returns a [`ProgressRequestBuilder`] that will display a progress bar
    /// when the request is sent. The progress bar shows:
    /// - Transfer speed (e.g., "45.2 MB/s")
    /// - Downloaded size / Total size (e.g., "45.2/100.0 MB")
    /// - ETA (estimated time remaining)
    /// - A visual progress bar
    ///
    /// When the content length is unknown (no `Content-Length` header),
    /// the progress bar falls back to a spinner animation.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use gilt::http::RequestBuilderProgress;
    ///
    /// # async fn example() -> reqwest::Result<()> {
    /// let response = reqwest::Client::new()
    ///     .get("https://example.com/large-file.zip")
    ///     .with_progress("Downloading large file")
    ///     .send()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    fn with_progress(self, description: &str) -> ProgressRequestBuilder;
}

impl RequestBuilderProgress for RequestBuilder {
    fn with_progress(self, description: &str) -> ProgressRequestBuilder {
        ProgressRequestBuilder {
            inner: self,
            description: description.to_string(),
        }
    }
}

// ---------------------------------------------------------------------------
// ProgressRequestBuilder
// ---------------------------------------------------------------------------

/// A wrapper around [`reqwest::RequestBuilder`] that adds progress tracking.
///
/// Created by calling [`with_progress`](RequestBuilderProgress::with_progress)
/// on a [`reqwest::RequestBuilder`]. The progress display starts when
/// [`send`](ProgressRequestBuilder::send) is called and automatically stops
/// when the response body has been fully received.
///
/// # Examples
///
/// ```rust,no_run
/// use gilt::http::RequestBuilderProgress;
///
/// # async fn example() -> reqwest::Result<()> {
/// let response = reqwest::Client::new()
///     .get("https://example.com/data.json")
///     .with_progress("Fetching data")
///     .send()
///     .await?;
/// # Ok(())
/// # }
/// ```
pub struct ProgressRequestBuilder {
    inner: RequestBuilder,
    description: String,
}

impl ProgressRequestBuilder {
    /// Send the request with progress tracking.
    ///
    /// This method sends the request and displays a progress bar while
    /// receiving the response body. The progress bar shows transfer speed,
    /// downloaded/total size, ETA, and a visual bar.
    ///
    /// Returns a [`ProgressResponse`] that can be used to read the response
    /// body with continued progress tracking.
    ///
    /// # Errors
    ///
    /// Returns a [`reqwest::Error`] if the request fails or if there are
    /// network errors.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use gilt::http::RequestBuilderProgress;
    ///
    /// # async fn example() -> reqwest::Result<()> {
    /// let response = reqwest::Client::new()
    ///     .get("https://example.com/file.zip")
    ///     .with_progress("Downloading")
    ///     .send()
    ///     .await?;
    ///
    /// // Read the body with progress tracking
    /// let bytes = response.bytes().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn send(self) -> Result<ProgressResponse> {
        let description = self.description.clone();
        let response = self.inner.send().await?;
        Ok(ProgressResponse::new(response, description))
    }
}

// ---------------------------------------------------------------------------
// ProgressResponse
// ---------------------------------------------------------------------------

/// A wrapper around [`reqwest::Response`] that provides progress tracking
/// while reading the response body.
///
/// Created by calling [`send`](ProgressRequestBuilder::send) on a
/// [`ProgressRequestBuilder`]. The progress display automatically starts
/// when the response is received and stops when the body is fully consumed.
///
/// # Examples
///
/// ```rust,no_run
/// use gilt::http::RequestBuilderProgress;
///
/// # async fn example() -> reqwest::Result<()> {
/// let response = reqwest::Client::new()
///     .get("https://example.com/file.zip")
///     .with_progress("Downloading")
///     .send()
///     .await?;
///
/// // Read all bytes with progress tracking
/// let bytes = response.bytes().await?;
/// println!("Downloaded {} bytes", bytes.len());
/// # Ok(())
/// # }
/// ```
pub struct ProgressResponse {
    inner: Option<Response>,
    progress: Option<Progress>,
    task_id: TaskId,
    total: Option<f64>,
    #[allow(dead_code)]
    description: String,
}

impl ProgressResponse {
    /// Create a new `ProgressResponse` from a [`reqwest::Response`].
    fn new(response: Response, description: String) -> Self {
        // Get content length if available
        let total = response.content_length().map(|n| n as f64);

        // Create progress display
        let mut progress = create_progress(total);
        let task_id = progress.add_task(&description, total);
        progress.start();

        Self {
            inner: Some(response),
            progress: Some(progress),
            task_id,
            total,
            description,
        }
    }

    /// Returns the status code of the response.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use gilt::http::RequestBuilderProgress;
    ///
    /// # async fn example() -> reqwest::Result<()> {
    /// let response = reqwest::Client::new()
    ///     .get("https://example.com/file.zip")
    ///     .with_progress("Downloading")
    ///     .send()
    ///     .await?;
    ///
    /// println!("Status: {}", response.status());
    /// # Ok(())
    /// # }
    /// ```
    pub fn status(&self) -> StatusCode {
        self.inner
            .as_ref()
            .map(|r| r.status())
            .unwrap_or(StatusCode::OK)
    }

    /// Returns the content length of the response if available.
    ///
    /// This is the value from the `Content-Length` header, if present.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use gilt::http::RequestBuilderProgress;
    ///
    /// # async fn example() -> reqwest::Result<()> {
    /// let response = reqwest::Client::new()
    ///     .get("https://example.com/file.zip")
    ///     .with_progress("Downloading")
    ///     .send()
    ///     .await?;
    ///
    /// if let Some(len) = response.content_length() {
    ///     println!("Content length: {} bytes", len);
    /// }
    ///
    /// let bytes = response.bytes().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn content_length(&self) -> Option<u64> {
        self.inner.as_ref().and_then(|r| r.content_length())
    }

    /// Read the response body as bytes with progress tracking.
    ///
    /// This method reads the entire response body into memory as [`Bytes`].
    /// The progress bar updates as data is received.
    ///
    /// # Errors
    ///
    /// Returns a [`reqwest::Error`] if there are network errors or if the
    /// body cannot be read.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use gilt::http::RequestBuilderProgress;
    ///
    /// # async fn example() -> reqwest::Result<()> {
    /// let response = reqwest::Client::new()
    ///     .get("https://example.com/file.zip")
    ///     .with_progress("Downloading")
    ///     .send()
    ///     .await?;
    ///
    /// let bytes = response.bytes().await?;
    /// println!("Downloaded {} bytes", bytes.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn bytes(mut self) -> Result<Bytes> {
        let result = self.collect_body().await;
        self.stop_progress();
        result
    }

    /// Read the response body as a string with progress tracking.
    ///
    /// This method reads the entire response body and converts it to a `String`.
    /// The progress bar updates as data is received.
    ///
    /// # Errors
    ///
    /// Returns a [`reqwest::Error`] if there are network errors or if the body
    /// cannot be read. Returns a `FromUtf8Error` if the body is not valid UTF-8.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use gilt::http::RequestBuilderProgress;
    ///
    /// # async fn example() -> reqwest::Result<()> {
    /// let response = reqwest::Client::new()
    ///     .get("https://api.example.com/data.json")
    ///     .with_progress("Fetching JSON")
    ///     .send()
    ///     .await?;
    ///
    /// let text = response.text().await?;
    /// println!("Response: {}", text);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn text(mut self) -> std::result::Result<String, TextError> {
        let bytes = self.collect_body().await.map_err(TextError::Reqwest)?;
        self.stop_progress();
        String::from_utf8(bytes.to_vec()).map_err(TextError::Utf8)
    }

    /// Parse the response body as JSON with progress tracking.
    ///
    /// This method reads the entire response body and deserializes it as JSON
    /// into the target type `T`. The progress bar updates as data is received.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The target type to deserialize into. Must implement [`serde::de::DeserializeOwned`].
    ///
    /// # Errors
    ///
    /// Returns a [`JsonError`] if there are network errors, if the body
    /// cannot be read, or if the JSON is invalid.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use serde::Deserialize;
    /// use gilt::http::RequestBuilderProgress;
    ///
    /// #[derive(Deserialize)]
    /// struct User {
    ///     name: String,
    ///     email: String,
    /// }
    ///
    /// # async fn example() -> reqwest::Result<()> {
    /// let user: User = reqwest::Client::new()
    ///     .get("https://api.example.com/user/1")
    ///     .with_progress("Loading user")
    ///     .send()
    ///     .await?
    ///     .json()
    ///     .await?;
    ///
    /// println!("User: {}", user.name);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn json<T: serde::de::DeserializeOwned>(
        mut self,
    ) -> std::result::Result<T, JsonError> {
        let bytes = self.collect_body().await.map_err(JsonError::Reqwest)?;
        self.stop_progress();
        serde_json::from_slice(&bytes).map_err(JsonError::Json)
    }

    /// Collect the response body into bytes with progress updates.
    async fn collect_body(&mut self) -> Result<Bytes> {
        use futures_util::StreamExt;

        // Take ownership of the inner response to consume the body
        let response = self.inner.take().expect("inner response already consumed");
        let mut body_stream = response.bytes_stream();
        let mut collected = Vec::new();

        while let Some(chunk) = body_stream.next().await {
            let chunk: Bytes = chunk?;
            collected.extend_from_slice(&chunk);

            // Update progress
            if let Some(ref mut progress) = self.progress {
                progress.advance(self.task_id, chunk.len() as f64);
                progress.refresh();
            }
        }

        Ok(Bytes::from(collected))
    }

    /// Stop the progress display.
    fn stop_progress(&mut self) {
        if let Some(mut progress) = self.progress.take() {
            // Mark task as complete
            if let Some(total) = self.total {
                progress.update(self.task_id, Some(total), None, None, None, None);
            }
            progress.stop();
        }
    }
}

impl Drop for ProgressResponse {
    fn drop(&mut self) {
        self.stop_progress();
    }
}

// ---------------------------------------------------------------------------
// Convenience functions
// ---------------------------------------------------------------------------

/// Perform a GET request with progress tracking.
///
/// This is a convenience function that creates a new [`reqwest::Client`],
/// sends a GET request to the specified URL, and returns a [`ProgressResponse`]
/// with progress tracking enabled.
///
/// # Arguments
///
/// * `url` - The URL to request.
///
/// # Errors
///
/// Returns a [`reqwest::Error`] if the request fails.
///
/// # Examples
///
/// ```rust,no_run
/// # async fn example() -> reqwest::Result<()> {
/// let response = gilt::http::get("https://api.example.com/data.json").await?;
/// let bytes = response.bytes().await?;
/// # Ok(())
/// # }
/// ```
pub async fn get(url: &str) -> Result<ProgressResponse> {
    Client::new()
        .get(url)
        .with_progress("Downloading")
        .send()
        .await
}

/// Perform a POST request with progress tracking.
///
/// This is a convenience function that creates a new [`reqwest::Client`],
/// sends a POST request to the specified URL, and returns a [`ProgressResponse`]
/// with progress tracking enabled.
///
/// # Arguments
///
/// * `url` - The URL to request.
///
/// # Errors
///
/// Returns a [`reqwest::Error`] if the request fails.
///
/// # Examples
///
/// ```rust,no_run
/// # async fn example() -> reqwest::Result<()> {
/// let response = gilt::http::post("https://api.example.com/upload").await?;
/// let bytes = response.bytes().await?;
/// # Ok(())
/// # }
/// ```
pub async fn post(url: &str) -> Result<ProgressResponse> {
    Client::new()
        .post(url)
        .with_progress("Uploading")
        .send()
        .await
}

/// Download a file from a URL to a local path with progress tracking.
///
/// This function downloads the content at the specified URL and saves it to
/// the given file path. A progress bar is displayed during the download.
///
/// # Arguments
///
/// * `url` - The URL to download from.
/// * `path` - The local file path to save to.
///
/// # Returns
///
/// Returns the number of bytes downloaded on success.
///
/// # Errors
///
/// Returns a [`DownloadError`] if the download fails or if there are I/O errors.
///
/// # Examples
///
/// ```rust,no_run
/// use std::path::Path;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let bytes_written = gilt::http::download(
///     "https://example.com/file.zip",
///     Path::new("file.zip")
/// ).await?;
/// println!("Downloaded {} bytes", bytes_written);
/// # Ok(())
/// # }
/// ```
pub async fn download(url: &str, path: &Path) -> std::result::Result<u64, DownloadError> {
    download_with_progress(url, path, "Downloading").await
}

/// Download a file from a URL to a local path with custom progress description.
///
/// This function is similar to [`download`], but allows you to specify a custom
/// description for the progress bar.
///
/// # Arguments
///
/// * `url` - The URL to download from.
/// * `path` - The local file path to save to.
/// * `description` - The description to show in the progress bar.
///
/// # Returns
///
/// Returns the number of bytes downloaded on success.
///
/// # Errors
///
/// Returns a [`DownloadError`] if the download fails or if there are I/O errors.
///
/// # Examples
///
/// ```rust,no_run
/// use std::path::Path;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let bytes_written = gilt::http::download_with_progress(
///     "https://example.com/large-file.zip",
///     Path::new("large-file.zip"),
///     "Downloading archive"
/// ).await?;
/// println!("Downloaded {} bytes", bytes_written);
/// # Ok(())
/// # }
/// ```
pub async fn download_with_progress(
    url: &str,
    path: &Path,
    description: &str,
) -> std::result::Result<u64, DownloadError> {
    use std::fs::File;
    use std::io::Write;

    let response = Client::new()
        .get(url)
        .with_progress(description)
        .send()
        .await?;

    let bytes = response.bytes().await.map_err(DownloadError::Http)?;
    let len = bytes.len() as u64;

    // Write to file
    let mut file = File::create(path)?;
    file.write_all(&bytes)?;

    Ok(len)
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

/// Create a progress display with appropriate columns for HTTP downloads.
///
/// This creates a progress bar with:
/// - Description column
/// - Progress bar column
/// - Downloaded/Total size column
/// - Transfer speed column
/// - Time remaining column
///
/// If `total` is `None`, the bar will show a spinner instead of a progress bar
/// (indeterminate mode).
fn create_progress(_total: Option<f64>) -> Progress {
    let columns: Vec<Box<dyn ProgressColumn>> = vec![
        Box::new(TextColumn::new("{task.description}")),
        Box::new(BarColumn::new()),
        Box::new(DownloadColumn::new()),
        Box::new(TransferSpeedColumn::new()),
        Box::new(TimeRemainingColumn::new()),
    ];

    Progress::new(columns)
        .with_auto_refresh(true)
        .with_refresh_per_second(10.0)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Most tests would require a mock HTTP server.
    // These are basic unit tests for the types.

    #[test]
    fn test_progress_request_builder_creation() {
        let builder = Client::new().get("https://example.com");
        let progress_builder = builder.with_progress("Test");
        assert_eq!(progress_builder.description, "Test");
    }

    #[test]
    fn test_create_progress_with_total() {
        let _progress = create_progress(Some(1000.0));
        // Progress should be created successfully
        // We can't easily inspect the internal state, but we can verify it doesn't panic
    }

    #[test]
    fn test_create_progress_without_total() {
        let _progress = create_progress(None);
        // Progress should be created successfully for indeterminate mode
    }
}
