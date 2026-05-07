//! Secure storage for JWT access and refresh tokens using the OS keychain.
//!
//! Tokens are stored under service name `"solitaire_quest_server"` with entry
//! keys `"{username}_access"` and `"{username}_refresh"`.
//!
//! On Linux this requires a running secret service (GNOME Keyring / KWallet).
//! If the keychain is unavailable, operations return
//! [`TokenError::KeychainUnavailable`] — callers should fall back to prompting
//! the user to log in again.
//!
//! Before calling any function in this module the application must initialise
//! the default keyring store exactly once at startup by calling
//! `keyring::use_native_store` (e.g. in `solitaire_app::main` before building
//! the Bevy `App`). If no default store is set, all operations in this module
//! will return [`TokenError::KeychainUnavailable`].
//!
//! # Android stub
//!
//! `keyring-core` cannot compile for the android target (its `rpassword`
//! transitive dep uses `libc::__errno_location`, which Android's bionic
//! doesn't expose). On Android every function in this module returns
//! [`TokenError::KeychainUnavailable`] so callers can detect the fallback
//! the same way they handle a Linux box without Secret Service. The
//! real Android backend will arrive in the Phase-Android round when we
//! wire Android Keystore via JNI.
//!
//! # Note: no unit tests — requires live OS keychain.

#[cfg(not(target_os = "android"))]
use keyring_core::Entry;
use thiserror::Error;

/// Errors that can occur when reading or writing tokens in the OS keychain.
#[derive(Debug, Error)]
pub enum TokenError {
    /// The OS keychain (secret service / keychain daemon) is not available.
    #[error("keychain unavailable: {0}")]
    KeychainUnavailable(String),
    /// No token was found in the keychain for the given username.
    #[error("token not found for user {0}")]
    NotFound(String),
    /// An unexpected keychain error occurred.
    #[error("keychain error: {0}")]
    Keyring(String),
}

/// Service name used to namespace all keychain entries for this application.
#[cfg(not(target_os = "android"))]
const SERVICE: &str = "solitaire_quest_server";

/// Map a `keyring_core::Error` to the appropriate `TokenError`.
#[cfg(not(target_os = "android"))]
fn map_keyring_err(err: keyring_core::Error, username: &str) -> TokenError {
    let msg = err.to_string();
    match err {
        keyring_core::Error::NoStorageAccess(_) | keyring_core::Error::NoDefaultStore => {
            TokenError::KeychainUnavailable(msg)
        }
        keyring_core::Error::NoEntry => TokenError::NotFound(username.to_string()),
        _ => TokenError::Keyring(msg),
    }
}

/// Store the access and refresh tokens for `username` in the OS keychain.
///
/// Any previously stored tokens for that username are overwritten.
#[cfg(not(target_os = "android"))]
pub fn store_tokens(
    username: &str,
    access_token: &str,
    refresh_token: &str,
) -> Result<(), TokenError> {
    Entry::new(SERVICE, &format!("{username}_access"))
        .map_err(|e| map_keyring_err(e, username))?
        .set_password(access_token)
        .map_err(|e| map_keyring_err(e, username))?;

    Entry::new(SERVICE, &format!("{username}_refresh"))
        .map_err(|e| map_keyring_err(e, username))?
        .set_password(refresh_token)
        .map_err(|e| map_keyring_err(e, username))?;

    Ok(())
}

/// Load the stored access token for `username` from the OS keychain.
///
/// Returns [`TokenError::NotFound`] if no token has been stored yet.
#[cfg(not(target_os = "android"))]
pub fn load_access_token(username: &str) -> Result<String, TokenError> {
    Entry::new(SERVICE, &format!("{username}_access"))
        .map_err(|e| map_keyring_err(e, username))?
        .get_password()
        .map_err(|e| map_keyring_err(e, username))
}

/// Load the stored refresh token for `username` from the OS keychain.
///
/// Returns [`TokenError::NotFound`] if no token has been stored yet.
#[cfg(not(target_os = "android"))]
pub fn load_refresh_token(username: &str) -> Result<String, TokenError> {
    Entry::new(SERVICE, &format!("{username}_refresh"))
        .map_err(|e| map_keyring_err(e, username))?
        .get_password()
        .map_err(|e| map_keyring_err(e, username))
}

/// Delete the stored access and refresh tokens for `username`.
///
/// Intended to be called on logout or account deletion. Missing entries are
/// silently ignored (the tokens are already gone, which is the desired state).
#[cfg(not(target_os = "android"))]
pub fn delete_tokens(username: &str) -> Result<(), TokenError> {
    match Entry::new(SERVICE, &format!("{username}_access"))
        .map_err(|e| map_keyring_err(e, username))?
        .delete_credential()
    {
        Ok(()) | Err(keyring_core::Error::NoEntry) => {}
        Err(e) => return Err(map_keyring_err(e, username)),
    }

    match Entry::new(SERVICE, &format!("{username}_refresh"))
        .map_err(|e| map_keyring_err(e, username))?
        .delete_credential()
    {
        Ok(()) | Err(keyring_core::Error::NoEntry) => {}
        Err(e) => return Err(map_keyring_err(e, username)),
    }

    Ok(())
}

// -------------------------------------------------------------------
// Android stub — same public API, always returns KeychainUnavailable.
// Lets `sync_client::*` compile unchanged on Android; the runtime
// effect is "session login required every launch", same as a Linux
// box without Secret Service.
// -------------------------------------------------------------------

#[cfg(target_os = "android")]
const ANDROID_STUB_MSG: &str = "android stub: keychain not yet wired (Phase-Android task)";

#[cfg(target_os = "android")]
pub fn store_tokens(
    _username: &str,
    _access_token: &str,
    _refresh_token: &str,
) -> Result<(), TokenError> {
    Err(TokenError::KeychainUnavailable(ANDROID_STUB_MSG.to_string()))
}

#[cfg(target_os = "android")]
pub fn load_access_token(_username: &str) -> Result<String, TokenError> {
    Err(TokenError::KeychainUnavailable(ANDROID_STUB_MSG.to_string()))
}

#[cfg(target_os = "android")]
pub fn load_refresh_token(_username: &str) -> Result<String, TokenError> {
    Err(TokenError::KeychainUnavailable(ANDROID_STUB_MSG.to_string()))
}

#[cfg(target_os = "android")]
pub fn delete_tokens(_username: &str) -> Result<(), TokenError> {
    Err(TokenError::KeychainUnavailable(ANDROID_STUB_MSG.to_string()))
}
