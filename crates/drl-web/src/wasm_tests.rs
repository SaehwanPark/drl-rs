//! In-browser WASM contracts for the boot, key, save, and quarantine surface.

use crate::{BrowserSession, SnapshotError};
use drl_protocol::{Command, Direction};
use wasm_bindgen_test::*;
use web_sys::window;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn key_contract_is_stable() {
  assert!(crate::key_command("ArrowUp").contains("Move"));
}

#[wasm_bindgen_test]
fn v3_storage_round_trip_and_rejection_quarantine_are_bounded() {
  let storage = window()
    .expect("browser window")
    .local_storage()
    .expect("localStorage access")
    .expect("localStorage available");
  storage
    .remove_item(crate::wasm::SAVE_STORAGE_KEY)
    .expect("remove active save");
  storage
    .remove_item(crate::wasm::REJECTED_SAVE_STORAGE_KEY)
    .expect("remove rejected save");

  let mut expected = BrowserSession::new().expect("fixed session");
  expected
    .submit(Command::Move(Direction::East))
    .expect("legal command");
  let token = expected.snapshot_token().expect("snapshot encoding");
  crate::wasm::storage::persist_session(&expected).expect("persist V3 snapshot");
  assert_eq!(
    crate::wasm::storage::read_persisted_session().expect("read V3 snapshot"),
    Some(token.clone())
  );

  let mut restored = BrowserSession::new().expect("fixed session");
  restored
    .restore_snapshot(&token)
    .expect("restore V3 snapshot");
  assert_eq!(restored, expected);

  let rejected_token = token.replace(":144:", ":127:");
  storage
    .set_item(crate::wasm::SAVE_STORAGE_KEY, &rejected_token)
    .expect("write rejected active save");
  let before = restored.clone();
  let error = restored
    .restore_snapshot(&rejected_token)
    .expect_err("incompatible snapshot must reject");
  assert_eq!(
    error,
    SnapshotError::UnsupportedGameplaySemantics {
      found: 127,
      expected: drl_protocol::CURRENT_GAMEPLAY_SEMANTICS_VERSION,
    }
  );
  assert_eq!(restored, before);
  crate::wasm::storage::quarantine_persisted_session(&rejected_token, &error)
    .expect("quarantine rejected snapshot");
  assert!(
    crate::wasm::storage::read_persisted_session()
      .expect("read cleared active save")
      .is_none()
  );
  let rejected_record = storage
    .get_item(crate::wasm::REJECTED_SAVE_STORAGE_KEY)
    .expect("read quarantine")
    .expect("quarantine record");
  assert!(rejected_record.contains("error=unsupported snapshot gameplay semantics"));
  storage
    .remove_item(crate::wasm::SAVE_STORAGE_KEY)
    .expect("cleanup active save");
  storage
    .remove_item(crate::wasm::REJECTED_SAVE_STORAGE_KEY)
    .expect("cleanup rejected save");
}
