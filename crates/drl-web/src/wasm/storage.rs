//! Browser localStorage session persistence shell. Tokens carry semantic
//! identities; mismatches are quarantined and never replayed.

use super::*;

fn browser_storage() -> Result<Storage, SnapshotError> {
  let window = web_sys::window()
    .ok_or_else(|| SnapshotError::Initialization("window unavailable".to_string()))?;
  window
    .local_storage()
    .map_err(|error| SnapshotError::Initialization(format!("localStorage unavailable: {error:?}")))?
    .ok_or_else(|| SnapshotError::Initialization("localStorage unavailable".to_string()))
}

pub(crate) fn persist_session(session: &BrowserSession) -> Result<(), SnapshotError> {
  let token = session.snapshot_token()?;
  browser_storage()?
    .set_item(SAVE_STORAGE_KEY, &token)
    .map_err(|error| SnapshotError::Initialization(format!("save failed: {error:?}")))
}

pub(crate) fn remove_persisted_session() -> Result<(), SnapshotError> {
  browser_storage()?
    .remove_item(SAVE_STORAGE_KEY)
    .map_err(|error| SnapshotError::Initialization(format!("clear failed: {error:?}")))
}

pub(crate) fn remove_rejected_session() -> Result<(), SnapshotError> {
  browser_storage()?
    .remove_item(REJECTED_SAVE_STORAGE_KEY)
    .map_err(|error| SnapshotError::Initialization(format!("quarantine clear failed: {error:?}")))
}

pub(crate) fn quarantine_persisted_session(
  token: &str,
  error: &SnapshotError,
) -> Result<(), SnapshotError> {
  let storage = browser_storage()?;
  let record = persistence::encode_quarantine_record(token, error);
  storage
    .set_item(REJECTED_SAVE_STORAGE_KEY, &record)
    .map_err(|storage_error| {
      SnapshotError::Initialization(format!("quarantine write failed: {storage_error:?}"))
    })?;
  storage
    .remove_item(SAVE_STORAGE_KEY)
    .map_err(|storage_error| {
      SnapshotError::Initialization(format!("active save clear failed: {storage_error:?}"))
    })
}

pub(crate) fn rejected_save_message(token: &str, error: &SnapshotError) -> String {
  match quarantine_persisted_session(token, error) {
    Ok(()) => format!(" Saved session ignored ({error}); rejected save quarantined."),
    Err(recovery_error) => {
      format!(" Saved session ignored ({error}); rejected save may remain ({recovery_error}).")
    }
  }
}

pub(crate) fn append_persistence_warning(status: String, warning: Option<String>) -> String {
  match warning {
    Some(warning) => format!("{status}{warning}"),
    None => status,
  }
}

pub(crate) fn save_after_command(session: &BrowserSession) -> Option<String> {
  let warning = persist_session(session).err().map(|error| {
    format!(" Save warning: current session was not persisted ({error}); use Save to retry.")
  });
  if warning.is_none() {
    if let Some(document) = web_sys::window().and_then(|window| window.document()) {
      clear_persistence_diagnostic(&document);
    }
  }
  warning
}

pub(crate) fn read_persisted_session() -> Result<Option<String>, SnapshotError> {
  browser_storage()?
    .get_item(SAVE_STORAGE_KEY)
    .map_err(|error| SnapshotError::Initialization(format!("load failed: {error:?}")))
}
