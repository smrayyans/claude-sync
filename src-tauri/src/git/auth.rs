use anyhow::{Context, Result};

const SERVICE: &str = "claude-sync";

/// Store a PAT token in the OS keychain
pub fn store_token(remote_url: &str, token: &str) -> Result<()> {
    let entry = keyring::Entry::new(SERVICE, remote_url)
        .with_context(|| "Failed to create keyring entry")?;
    entry
        .set_password(token)
        .with_context(|| "Failed to store token in keychain")?;
    Ok(())
}

/// Retrieve a PAT token from the OS keychain
pub fn get_token(remote_url: &str) -> Result<String> {
    let entry = keyring::Entry::new(SERVICE, remote_url)
        .with_context(|| "Failed to create keyring entry")?;
    let token = entry
        .get_password()
        .with_context(|| "Failed to retrieve token from keychain")?;
    Ok(token)
}

/// Delete a stored token
pub fn delete_token(remote_url: &str) -> Result<()> {
    let entry = keyring::Entry::new(SERVICE, remote_url)
        .with_context(|| "Failed to create keyring entry")?;
    entry
        .delete_credential()
        .with_context(|| "Failed to delete token from keychain")?;
    Ok(())
}

/// Build git2 RemoteCallbacks with PAT authentication
pub fn build_callbacks(token: &str) -> git2::RemoteCallbacks<'_> {
    let token = token.to_string();
    let mut callbacks = git2::RemoteCallbacks::new();
    callbacks.credentials(move |_url, username, _allowed| {
        git2::Cred::userpass_plaintext(username.unwrap_or("git"), &token)
    });
    callbacks
}
