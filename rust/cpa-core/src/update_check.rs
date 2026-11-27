//! Update check: compares the running version with the latest release on GitHub.
//!
//! Port of `application/helper_classes/updateCheck.py`. Only the public release information is requested; nothing
//! about the CPA or the user is sent. The request is injectable (see [`Fetch`]) so the tests never use the network.

use std::fmt;
use std::sync::LazyLock;
use std::time::Duration;

use regex::Regex;
use ureq::tls::{RootCerts, TlsConfig, TlsProvider};

/// GitHub never returns a prerelease (release candidate, nightly build) or draft as the latest release, so those
/// are never offered as an update.
pub const LATEST_RELEASE_API_URL: &str = "https://api.github.com/repos/gidie12/cpaEditor/releases/latest";
/// Fixed address of the release page: nothing from the response of the update check is ever opened.
pub const LATEST_RELEASE_PAGE_URL: &str = "https://github.com/gidie12/cpaEditor/releases/latest";
/// The Windows executable that the release workflow attaches to every release.
pub const LATEST_RELEASE_DOWNLOAD_URL: &str =
    "https://github.com/gidie12/cpaEditor/releases/latest/download/CpaEditor.exe";
/// Headers of the request to the GitHub API.
pub const REQUEST_HEADERS: [(&str, &str); 2] =
    [("Accept", "application/vnd.github+json"), ("User-Agent", "cpaEditor-update-check")];
/// Timeout of the request when the caller has no preference.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

/// A dotted number, optionally followed by a prerelease label ('-rc.1') and build information ('+nightly.20261029').
static VERSION_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^v?(\d+(?:\.\d+)*)(-[0-9A-Za-z.-]+)?(\+[0-9A-Za-z.-]+)?$").expect("valid version pattern")
});

/// A text that is not a version number.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("Not a version number: '{0}'")]
pub struct VersionError(pub String);

/// Converts a version such as 'v1.0.2' or '1.0.2' to the numeric parts that can be compared, for example [1, 0, 2].
///
/// A prerelease label ('1.1.0-rc.1') and build information ('1.0.5+nightly.20261029.abc1234') are allowed but not
/// part of the result; see [`is_prerelease`].
pub fn parse_version(version: &str) -> Result<Vec<u64>, VersionError> {
    let captures = VERSION_PATTERN.captures(version.trim()).ok_or_else(|| VersionError(version.to_string()))?;
    captures[1].split('.').map(|part| part.parse::<u64>().map_err(|_| VersionError(version.to_string()))).collect()
}

/// Tells whether a version has a prerelease label, for example '1.1.0-rc.1'. Build information alone
/// ('1.0.5+nightly.20261029') is not a prerelease label: a nightly build is the released version plus changes.
pub fn is_prerelease(version: &str) -> bool {
    VERSION_PATTERN.captures(version.trim()).is_some_and(|captures| captures.get(2).is_some())
}

/// Tells whether `latest_version` is newer than `current_version`; '1.1' and '1.1.0' are the same version.
///
/// A release is newer than a prerelease of the same version: '1.1.0' is newer than '1.1.0-rc.1'.
pub fn is_newer(latest_version: &str, current_version: &str) -> Result<bool, VersionError> {
    let mut latest = parse_version(latest_version)?;
    let mut current = parse_version(current_version)?;
    let length = latest.len().max(current.len());
    latest.resize(length, 0);
    current.resize(length, 0);
    if latest == current {
        return Ok(is_prerelease(current_version) && !is_prerelease(latest_version));
    }
    Ok(latest > current)
}

/// Why a request did not deliver a body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FetchError {
    /// The server answered with this HTTP status instead of the release information.
    Status(u16),
    /// The host name could not be resolved; the text is the reason given by the system.
    HostNotFound(String),
    /// Any other failure to get an answer: connection, proxy, TLS, timeout.
    Network(String),
}

impl fmt::Display for FetchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FetchError::Status(code) => write!(f, "HTTP Error {code}"),
            FetchError::HostNotFound(reason) | FetchError::Network(reason) => f.write_str(reason),
        }
    }
}

impl std::error::Error for FetchError {}

/// A request: `(url, headers, timeout)` to the body of a response with status 200.
pub type Fetch<'a> = dyn Fn(&str, &[(&str, &str)], Duration) -> Result<Vec<u8>, FetchError> + 'a;

/// Why the latest version could not be determined.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum UpdateError {
    /// The request failed; `windows_error` is the failure of the retry with the internet settings of Windows.
    #[error("{}", describe_network_error(.error, .windows_error.as_ref()))]
    Network { error: FetchError, windows_error: Option<FetchError> },
    /// GitHub answered, but not with release information.
    #[error("{0}")]
    Response(String),
    /// The tag of the latest release is not a version number.
    #[error(transparent)]
    Version(#[from] VersionError),
}

/// The configuration of the HTTP client: TLS of the operating system (native-tls), verified with the certificates
/// trusted by the operating system, so a company network with its own root certificate works. The proxy of the
/// environment (HTTPS_PROXY) is used, as in the default configuration of ureq.
pub fn agent_config(timeout: Duration) -> ureq::config::Config {
    ureq::Agent::config_builder()
        .timeout_global(Some(timeout))
        .tls_config(
            TlsConfig::builder().provider(TlsProvider::NativeTls).root_certs(RootCerts::PlatformVerifier).build(),
        )
        .build()
}

/// Tells whether an I/O error of the connection is a name that could not be resolved (getaddrinfo).
fn is_unresolved(error: &std::io::Error) -> bool {
    // WSAHOST_NOT_FOUND, WSATRY_AGAIN, WSANO_RECOVERY, WSANO_DATA on Windows; the text of the standard library elsewhere
    matches!(error.raw_os_error(), Some(11001..=11004))
        || error.to_string().starts_with("failed to lookup address information")
}

fn to_fetch_error(error: ureq::Error) -> FetchError {
    match error {
        ureq::Error::StatusCode(code) => FetchError::Status(code),
        ureq::Error::HostNotFound => FetchError::HostNotFound(error.to_string()),
        ureq::Error::Io(ref io) if is_unresolved(io) => FetchError::HostNotFound(io.to_string()),
        other => FetchError::Network(other.to_string()),
    }
}

/// Requests a URL directly (or through HTTPS_PROXY) with TLS verified against the certificates of the system.
pub fn fetch_direct(url: &str, headers: &[(&str, &str)], timeout: Duration) -> Result<Vec<u8>, FetchError> {
    let agent = ureq::Agent::new_with_config(agent_config(timeout));
    let mut request = agent.get(url);
    for (name, value) in headers {
        request = request.header(*name, *value);
    }
    let mut response = request.call().map_err(to_fetch_error)?;
    response.body_mut().read_to_vec().map_err(to_fetch_error)
}

/// Reads the tag of the release from the answer of the GitHub API.
fn read_tag_name(body: &[u8]) -> Result<String, UpdateError> {
    let release: serde_json::Value =
        serde_json::from_slice(body).map_err(|error| UpdateError::Response(error.to_string()))?;
    release
        .get("tag_name")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| UpdateError::Response("The answer of GitHub has no tag_name".to_string()))
}

/// Asks GitHub for the tag of the latest release, for example 'v1.0.2', with `fetch`.
///
/// A request that cannot get out is repeated with `fallback` (the internet settings of Windows), but not when GitHub
/// itself answered (rate limit, not found); 407 is the proxy asking for a login and is repeated.
pub fn get_latest_version_with(
    fetch: &Fetch<'_>,
    fallback: Option<&Fetch<'_>>,
    timeout: Duration,
) -> Result<String, UpdateError> {
    let body = match fetch(LATEST_RELEASE_API_URL, &REQUEST_HEADERS, timeout) {
        Ok(body) => body,
        Err(error) => {
            let answered = matches!(error, FetchError::Status(code) if code != 407);
            let fallback = match fallback {
                Some(fallback) if !answered => fallback,
                _ => return Err(UpdateError::Network { error, windows_error: None }),
            };
            match fallback(LATEST_RELEASE_API_URL, &REQUEST_HEADERS, timeout) {
                Ok(body) => body,
                Err(windows_error) => return Err(UpdateError::Network { error, windows_error: Some(windows_error) }),
            }
        }
    };
    read_tag_name(&body)
}

/// Asks GitHub for the tag of the latest release. On Windows a request that cannot get out directly is repeated
/// with the internet settings of Windows, so the check also works behind a company proxy.
pub fn get_latest_version(timeout: Duration) -> Result<String, UpdateError> {
    #[cfg(windows)]
    let fallback: Option<&Fetch<'_>> = Some(&fetch_with_windows_internet_settings);
    #[cfg(not(windows))]
    let fallback: Option<&Fetch<'_>> = None;
    get_latest_version_with(&fetch_direct, fallback, timeout)
}

/// Explains a failed request in words a user can act on.
///
/// A name that cannot be resolved ('getaddrinfo failed' on Windows) means there is no internet connection, or the
/// network only reaches the internet through a proxy that is not configured.
pub fn describe_network_error(error: &FetchError, windows_error: Option<&FetchError>) -> String {
    let mut message = match error {
        FetchError::HostNotFound(reason) => format!(
            "api.github.com could not be found ({reason}): there is no internet connection, or the network needs a \
             proxy (set HTTPS_PROXY). The latest version is at {LATEST_RELEASE_PAGE_URL}"
        ),
        other => other.to_string(),
    };
    if let Some(windows_error) = windows_error {
        message.push_str(&format!(" (the internet settings of Windows did not work either: {windows_error})"));
    }
    message
}

/// Result of the update check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateResult {
    /// The running version.
    pub current: String,
    /// The latest release without 'v', when the check succeeded.
    pub latest: Option<String>,
    /// Whether the latest release is newer than the running version; false when the check failed.
    pub update_available: bool,
    /// Why the check failed.
    pub error: Option<String>,
}

/// Compares the running version with the outcome of [`get_latest_version`].
pub fn check_for_update_with(current_version: &str, latest_version: Result<String, UpdateError>) -> UpdateResult {
    let mut result =
        UpdateResult { current: current_version.to_string(), latest: None, update_available: false, error: None };
    let outcome = latest_version.and_then(|latest| Ok((is_newer(&latest, current_version)?, latest)));
    match outcome {
        Ok((update_available, latest)) => {
            result.update_available = update_available;
            result.latest = Some(latest.trim_start_matches('v').to_string());
        }
        Err(error) => result.error = Some(error.to_string()),
    }
    result
}

/// Compares the running version with the latest release on GitHub.
pub fn check_for_update(current_version: &str, timeout: Duration) -> UpdateResult {
    check_for_update_with(current_version, get_latest_version(timeout))
}

/// Requests a URL through WinINet, the internet layer of Windows itself.
///
/// WinINet uses the proxy settings of the user exactly like the browser does: an automatic configuration script
/// (PAC), automatic detection and a proxy that asks for the Windows account. ureq knows none of these.
#[cfg(windows)]
pub fn fetch_with_windows_internet_settings(
    url: &str,
    headers: &[(&str, &str)],
    timeout: Duration,
) -> Result<Vec<u8>, FetchError> {
    use std::ffi::c_void;
    use std::ptr::{null, null_mut};
    use windows_sys::Win32::Networking::WinInet::{
        HttpQueryInfoW, InternetCloseHandle, InternetOpenUrlW, InternetOpenW, InternetReadFile, InternetSetOptionW,
        HTTP_QUERY_FLAG_NUMBER, HTTP_QUERY_STATUS_CODE, INTERNET_FLAG_NO_CACHE_WRITE, INTERNET_FLAG_NO_COOKIES,
        INTERNET_FLAG_NO_UI, INTERNET_FLAG_RELOAD, INTERNET_FLAG_SECURE, INTERNET_OPEN_TYPE_PRECONFIG,
        INTERNET_OPTION_CONNECT_TIMEOUT, INTERNET_OPTION_RECEIVE_TIMEOUT,
    };

    /// Closes a WinINet handle when it goes out of scope.
    struct Handle(*mut c_void);
    impl Drop for Handle {
        fn drop(&mut self) {
            // SAFETY: the handle was returned by WinINet, is not null and is closed only here.
            unsafe { InternetCloseHandle(self.0) };
        }
    }
    fn wide(text: &str) -> Vec<u16> {
        text.encode_utf16().chain(std::iter::once(0)).collect()
    }
    // Read right after the failing call, before any handle is closed
    fn last_error(call: &str) -> FetchError {
        let code = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
        FetchError::Network(format!("{call} failed with WinINet error {code}"))
    }

    let user_agent = wide(headers.iter().find(|(name, _)| *name == "User-Agent").map_or("", |(_, value)| *value));
    // SAFETY: user_agent is a null-terminated UTF-16 string that outlives the call; null proxy arguments are allowed.
    let session = unsafe { InternetOpenW(user_agent.as_ptr(), INTERNET_OPEN_TYPE_PRECONFIG, null(), null(), 0) };
    if session.is_null() {
        return Err(last_error("InternetOpenW"));
    }
    let session = Handle(session);

    let milliseconds = u32::try_from(timeout.as_millis()).unwrap_or(u32::MAX);
    for option in [INTERNET_OPTION_CONNECT_TIMEOUT, INTERNET_OPTION_RECEIVE_TIMEOUT] {
        // SAFETY: the buffer is a u32 of the given size, as these options expect.
        unsafe {
            InternetSetOptionW(
                session.0,
                option,
                &milliseconds as *const u32 as *const c_void,
                std::mem::size_of::<u32>() as u32,
            )
        };
    }

    let header_lines: String = headers
        .iter()
        .filter(|(name, _)| *name != "User-Agent")
        .map(|(name, value)| format!("{name}: {value}\r\n"))
        .collect();
    let header_lines = wide(&header_lines);
    let url = wide(url);
    // A fresh answer over TLS, without cookies and without dialogs
    let flags = INTERNET_FLAG_RELOAD
        | INTERNET_FLAG_NO_CACHE_WRITE
        | INTERNET_FLAG_SECURE
        | INTERNET_FLAG_NO_COOKIES
        | INTERNET_FLAG_NO_UI;
    // SAFETY: url and header_lines are null-terminated UTF-16 strings that outlive the call; the length excludes the
    // terminating null.
    let response = unsafe {
        InternetOpenUrlW(session.0, url.as_ptr(), header_lines.as_ptr(), (header_lines.len() - 1) as u32, flags, 0)
    };
    if response.is_null() {
        return Err(last_error("InternetOpenUrlW"));
    }
    let response = Handle(response);

    let mut status: u32 = 0;
    let mut size = std::mem::size_of::<u32>() as u32;
    // SAFETY: with HTTP_QUERY_FLAG_NUMBER the buffer is a u32 of the given size.
    let queried = unsafe {
        HttpQueryInfoW(
            response.0,
            HTTP_QUERY_STATUS_CODE | HTTP_QUERY_FLAG_NUMBER,
            &mut status as *mut u32 as *mut c_void,
            &mut size,
            null_mut(),
        )
    };
    if queried == 0 {
        return Err(last_error("HttpQueryInfoW"));
    }
    if status != 200 {
        return Err(FetchError::Status(u16::try_from(status).unwrap_or(u16::MAX)));
    }

    let mut body = Vec::new();
    let mut buffer = [0u8; 8192];
    loop {
        let mut read: u32 = 0;
        // SAFETY: the buffer is writable for its full length and read receives the number of bytes written.
        let ok =
            unsafe { InternetReadFile(response.0, buffer.as_mut_ptr() as *mut c_void, buffer.len() as u32, &mut read) };
        if ok == 0 {
            return Err(last_error("InternetReadFile"));
        }
        if read == 0 {
            return Ok(body);
        }
        body.extend_from_slice(&buffer[..read as usize]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    #[test]
    fn test_parse_version() {
        assert_eq!(parse_version("v1.0.2").unwrap(), vec![1, 0, 2]);
        assert_eq!(parse_version("1.10").unwrap(), vec![1, 10]);
        assert_eq!(parse_version(crate::VERSION).unwrap(), parse_version(&format!("v{}", crate::VERSION)).unwrap());
        assert_eq!(parse_version("v1.1.0-rc.1").unwrap(), vec![1, 1, 0]);
        assert_eq!(parse_version("1.0.5+nightly.20261029.abc1234").unwrap(), vec![1, 0, 5]);
    }

    #[test]
    fn test_is_prerelease() {
        assert!(is_prerelease("v1.1.0-rc.1"));
        assert!(is_prerelease("1.1.0-beta+build.5"));
        assert!(!is_prerelease("1.0.5+nightly.20261029.abc1234"));
        assert!(!is_prerelease("v1.0.5"));
    }

    #[test]
    fn test_parse_version_invalid() {
        for version in ["", "latest", "1.0-", "v1..2", "1.0+", "beta-1.0", "99999999999999999999"] {
            assert_eq!(parse_version(version), Err(VersionError(version.to_string())), "{version}");
        }
        assert_eq!(VersionError("nightly".to_string()).to_string(), "Not a version number: 'nightly'");
    }

    #[test]
    fn test_is_newer() {
        assert!(is_newer("v1.0.10", "1.0.9").unwrap());
        assert!(is_newer("v2.0", "1.9.9").unwrap());
        assert!(!is_newer("v1.1", "1.1.0").unwrap());
        assert!(!is_newer("v1.0.2", "1.1.0").unwrap());
    }

    #[test]
    fn test_is_newer_with_prerelease_and_nightly() {
        // A release candidate gets its release, a nightly build only the next release
        assert!(is_newer("v1.1.0", "1.1.0-rc.1").unwrap());
        assert!(!is_newer("v1.0.5", "1.1.0-rc.1").unwrap());
        assert!(!is_newer("v1.0.5", "1.0.5+nightly.20261029.abc1234").unwrap());
        assert!(is_newer("v1.0.6", "1.0.5+nightly.20261029.abc1234").unwrap());
    }

    #[test]
    fn test_get_latest_version_reads_tag() {
        let calls = RefCell::new(Vec::new());
        let fetch = |url: &str, headers: &[(&str, &str)], timeout: Duration| {
            let headers: Vec<(String, String)> =
                headers.iter().map(|(name, value)| (name.to_string(), value.to_string())).collect();
            calls.borrow_mut().push((url.to_string(), headers, timeout));
            Ok(br#"{"tag_name": "v1.2.3", "html_url": "https://example.com"}"#.to_vec())
        };
        assert_eq!(get_latest_version_with(&fetch, None, Duration::from_secs(3)).unwrap(), "v1.2.3");
        let calls = calls.borrow();
        assert_eq!(calls.len(), 1);
        let (url, headers, timeout) = &calls[0];
        assert_eq!(url, "https://api.github.com/repos/gidie12/cpaEditor/releases/latest");
        let expected = [("Accept", "application/vnd.github+json"), ("User-Agent", "cpaEditor-update-check")];
        let expected: Vec<(String, String)> =
            expected.iter().map(|(name, value)| (name.to_string(), value.to_string())).collect();
        assert_eq!(headers, &expected);
        assert_eq!(*timeout, Duration::from_secs(3));
    }

    #[test]
    fn test_tls_is_verified_with_system_certificates() {
        // Replaces the check of verify_mode and check_hostname of the Python test
        let config = agent_config(Duration::from_secs(3));
        let tls = config.tls_config();
        assert_eq!(tls.provider(), TlsProvider::NativeTls);
        assert!(matches!(tls.root_certs(), RootCerts::PlatformVerifier));
        assert!(!tls.disable_verification());
        assert!(tls.use_sni());
        assert!(config.http_status_as_error());
    }

    #[test]
    fn test_get_latest_version_invalid_answer() {
        for body in [&b"not json"[..], br#"{"name": "v1.2.3"}"#, br#"{"tag_name": 5}"#] {
            let fetch = |_: &str, _: &[(&str, &str)], _: Duration| Ok(body.to_vec());
            assert!(matches!(get_latest_version_with(&fetch, None, DEFAULT_TIMEOUT), Err(UpdateError::Response(_))));
        }
    }

    #[test]
    fn test_check_for_update_newer_version() {
        assert_eq!(
            check_for_update_with("1.1.0", Ok("v1.2.0".to_string())),
            UpdateResult {
                current: "1.1.0".to_string(),
                latest: Some("1.2.0".to_string()),
                update_available: true,
                error: None
            }
        );
    }

    #[test]
    fn test_check_for_update_up_to_date() {
        assert!(!check_for_update_with("1.1.0", Ok("v1.1.0".to_string())).update_available);
    }

    #[test]
    fn test_check_for_update_reports_error() {
        let no_network =
            UpdateError::Network { error: FetchError::Network("no network".to_string()), windows_error: None };
        assert_eq!(
            check_for_update_with("1.1.0", Err(no_network)),
            UpdateResult {
                current: "1.1.0".to_string(),
                latest: None,
                update_available: false,
                error: Some("no network".to_string())
            }
        );
        let result = check_for_update_with("1.1.0", Ok("nightly".to_string()));
        assert_eq!(result.error.as_deref(), Some("Not a version number: 'nightly'"));
        assert_eq!(result.latest, None);
    }

    #[test]
    fn test_get_latest_version_falls_back_to_windows_internet_settings() {
        let unresolved = FetchError::HostNotFound("[Errno 11001] getaddrinfo failed".to_string());
        let rate_limited = FetchError::Status(403);
        let proxy_login = FetchError::Status(407);
        let fallback_calls = RefCell::new(Vec::new());
        let fallback_error: RefCell<Option<FetchError>> = RefCell::new(None);
        let fallback = |url: &str, _: &[(&str, &str)], timeout: Duration| {
            fallback_calls.borrow_mut().push((url.to_string(), timeout));
            match fallback_error.borrow().clone() {
                Some(error) => Err(error),
                None => Ok(br#"{"tag_name": "v1.2.3"}"#.to_vec()),
            }
        };
        let failing = |error: &FetchError| {
            let error = error.clone();
            move |_: &str, _: &[(&str, &str)], _: Duration| -> Result<Vec<u8>, FetchError> { Err(error.clone()) }
        };

        // Windows: the fallback is given
        for error in [&unresolved, &proxy_login] {
            let result = get_latest_version_with(&failing(error), Some(&fallback), Duration::from_secs(3));
            assert_eq!(result.unwrap(), "v1.2.3");
        }
        assert_eq!(
            fallback_calls.borrow().last().unwrap(),
            &(LATEST_RELEASE_API_URL.to_string(), Duration::from_secs(3))
        );

        // An answer of GitHub itself is not retried
        fallback_calls.borrow_mut().clear();
        let result = get_latest_version_with(&failing(&rate_limited), Some(&fallback), DEFAULT_TIMEOUT);
        assert_eq!(result, Err(UpdateError::Network { error: rate_limited.clone(), windows_error: None }));
        assert!(fallback_calls.borrow().is_empty());

        // When Windows cannot get out either, both errors are reported
        *fallback_error.borrow_mut() = Some(FetchError::Network("no route".to_string()));
        let latest = get_latest_version_with(&failing(&unresolved), Some(&fallback), DEFAULT_TIMEOUT);
        let error_message = check_for_update_with("1.1.0", latest).error.unwrap();
        assert!(error_message.contains("HTTPS_PROXY"), "{error_message}");
        assert!(error_message.contains("no route"), "{error_message}");

        // Other systems have no such settings: get_latest_version passes no fallback there
        fallback_calls.borrow_mut().clear();
        let result = get_latest_version_with(&failing(&unresolved), None, DEFAULT_TIMEOUT);
        assert_eq!(result, Err(UpdateError::Network { error: unresolved.clone(), windows_error: None }));
        assert!(fallback_calls.borrow().is_empty());
    }

    #[test]
    fn test_describe_network_error() {
        let unresolved = FetchError::HostNotFound("[Errno 11001] getaddrinfo failed".to_string());
        let message = describe_network_error(&unresolved, None);
        assert!(message.contains("getaddrinfo failed"));
        assert!(message.contains("HTTPS_PROXY"));
        assert!(message.contains(LATEST_RELEASE_PAGE_URL));
        let latest = Err(UpdateError::Network { error: unresolved, windows_error: None });
        assert_eq!(check_for_update_with("1.1.0", latest).error.unwrap(), message);
        let refused = FetchError::Network("Connection refused (os error 61)".to_string());
        assert_eq!(describe_network_error(&refused, None), refused.to_string());
        assert_eq!(describe_network_error(&FetchError::Status(403), None), "HTTP Error 403");
    }

    #[test]
    fn test_unresolved_name_is_recognized() {
        use std::io::{Error, ErrorKind};
        assert_eq!(to_fetch_error(ureq::Error::HostNotFound), FetchError::HostNotFound("host not found".to_string()));
        let windows = Error::from_raw_os_error(11001);
        assert!(matches!(to_fetch_error(ureq::Error::Io(windows)), FetchError::HostNotFound(_)));
        let unix = Error::new(ErrorKind::Other, "failed to lookup address information: nodename nor servname provided");
        assert!(matches!(to_fetch_error(ureq::Error::Io(unix)), FetchError::HostNotFound(_)));
        let refused = Error::from(ErrorKind::ConnectionRefused);
        assert!(matches!(to_fetch_error(ureq::Error::Io(refused)), FetchError::Network(_)));
        assert_eq!(to_fetch_error(ureq::Error::StatusCode(404)), FetchError::Status(404));
        assert!(matches!(to_fetch_error(ureq::Error::ConnectProxyFailed("407".to_string())), FetchError::Network(_)));
    }
}
