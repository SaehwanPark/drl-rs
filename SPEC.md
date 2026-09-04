# Specification

Last reviewed: 2026-09-03
Current project version: `0.2.342`
Audited starting checkpoint: `main` at `85e50c4` (M8 modular browser shell,
PR #457, and canonical documentation reconciliation)
Delivery checkpoint: `main` merge commit `85e50c4` (PR #457, merged)

The [Roadmap](docs/DRL-RS_Project_Roadmap.md) owns milestone scope, ordering,
and progress. [`docs/steering/current-priorities.md`](docs/steering/current-priorities.md)
constrains slice selection while its stop gates remain open. This file expands
**exactly one active implementation slice**. Delivered history belongs in the
roadmap, changelog, evidence notes, and Git rather than accumulating here.

## 1. Status vocabulary

- `[x]` — **Delivered and verified**: supported by checked implementation and
  evidence.
- `[ ]` — **Open**: required by the active slice and not yet delivered.
- `NOT_RUN` — **Environment unavailable**: prerequisites were unavailable; no
  pass or failure is inferred.
- `INCONCLUSIVE` — **Evidence unresolved**: available evidence cannot support
  the claim.

## 2. Active implementation slice: Linux and Fedora CI coverage

Slice status: **open** — the jobs and the Fedora development-host script exist on
branch `codex/linux-ci-checks`; the hosted runs and the independent review in §2.8
are not closed.

### 2.1 Objective

Close the platform-track step 2 of
[`docs/steering/audit-2026-09-02.md`](steering/audit-2026-09-02.md): give CI a
complete Linux repository verification path (audit §5 MUST) and add a Fedora 43
development-host job that carries only the evidence the existing jobs cannot
(audit §5 SHOULD). This is the coverage prerequisite for the native frontend
boundary and the `drl-desktop` slice, so that native work lands on platforms CI
actually exercises.

### 2.2 Audited starting point

At audited starting revision `85e50c4` (version `0.2.341`):

- `.github/workflows/ci.yml` declared two jobs: `Repository checks` on
  `macos-latest` running `sh scripts/check-repository.sh`, and `WASM browser
  checks` on `ubuntu-latest` running `scripts/check-web.sh` plus the release
  bundle, manifest, rights, and detached-signing checks.
- No job ran the repository contract suite on Linux, and no job reproduced the
  Fedora development host that every recent slice was verified on locally.
- `scripts/test-branch-protection.sh` asserts the required contexts
  `Repository checks`, `WASM browser checks`, and `Review policy`.
- `scripts/check-version.sh` treats `.sh` files as code and `.github/**` and
  `.yml` as non-code, so adding a shell script requires exactly one patch
  transition while a workflow-only change would not.
- The local development host is Fedora 43 (GNOME/Mutter Wayland, Mesa Vulkan),
  but nothing in CI reproduced that environment, and no repository script
  recorded whether the native-adjacent crates need system packages.

### 2.3 Scope and ownership

- `.github/workflows/ci.yml`: add `check-linux` (`Repository checks (Linux)` on
  `ubuntu-latest`, mirroring the macOS job including `DRL_VERSION_BASE`) and
  `fedora-dev` (`Fedora 43 development host`, a `fedora:43` container that the
  Ubuntu runner launches, so `actions/checkout` keeps using the runner's own Node
  runtime).
- `scripts/check-fedora-dev.sh` (new): record host and toolchain identity; check
  `drl-render`, `drl-audio`, and `drl-web` natively; run the `drl-core` and
  `drl-protocol` test contracts; probe `/dev/dri`, `libvulkan`, and
  `WAYLAND_DISPLAY` and print `gpu_and_wayland_acceptance=NOT_RUN`. It installs
  nothing, so a missing prerequisite is a host-provisioning fact for the workflow,
  not a hidden dependency of the script.
- `VERSION`, `Cargo.toml`, `Cargo.lock`: one patch transition `0.2.341` to
  `0.2.342`, required because a `.sh` file is added.
- `CHANGELOG.md`, `ARCHITECTURE.md`, `docs/DRL-RS_Project_Roadmap.md`,
  `docs/steering/current-priorities.md`, and this slice: CI coverage and the
  platform-track step record, including the PR #457 merge checkpoint.
- Unchanged: every crate source, the browser boundary scripts, the deterministic
  kernel, the replay wire and RNG semantics, and the macOS/WASM jobs.

### 2.4 Review and branch contract

- Branch `codex/linux-ci-checks` from `main` at `85e50c4`; one PR carries the
  whole slice.
- `Review policy` passes for this slice: it changes no protected review path, so
  no non-self review is demanded. The solo-maintainer `enforce_admins=false`
  exception recorded in `CHANGELOG.md` and the roadmap checkpoints was needed for
  PR #457, whose `crates/drl-web/**` changes are protected.
- The two new jobs are informative at first: they join the required-context list
  only through an explicit branch-protection change, which this slice records
  rather than assumes.

### 2.5 Acceptance criteria

- [x] `sh scripts/check-fedora-dev.sh` exits `0` in a clean `fedora:43` container
  with the Fedora toolchain packages `git`, `which`, `rust`, `cargo`, `clippy`, and
  `rustfmt` installed: no additional project-specific native-library package was
  required, `drl-core` and `drl-protocol` report 760 passing assertions across 42
  test binaries, and the probe prints
  `dri=absent vulkan=library-absent wayland_session=absent
  gpu_and_wayland_acceptance=NOT_RUN`.
- [x] The proven container invocation is the exact invocation the workflow uses
  (same bind mount, `--workdir /src`, and `CARGO_TARGET_DIR` inside the container).
- [x] `.github/workflows/ci.yml` parses and declares the jobs `check`,
  `check-linux`, `fedora-dev`, and `web`.
- [x] Exactly one patch version transition `0.2.341` to `0.2.342`, consistent in
  `VERSION`, `Cargo.toml`, and `Cargo.lock` (`scripts/check-version.sh`).
- [x] `sh scripts/check-repository.sh` exits `0` on the Fedora 43 host against
  `c308167` plus the document-only edits in this ledger's commit.
- [ ] Hosted `Repository checks (Linux)` and `Fedora 43 development host` pass on
  the commit that merges. Both pass on `c308167` (`Repository checks (Linux)` in
  1m7s with 59 test-binary summaries and 1088 assertions, `Fedora 43 development
  host` in 52s) and `Review policy` also passes here; the merging commit's own run
  is still required.
- [ ] An attributable independent review returns `pass` on this slice.

### 2.6 Non-goals

- No Fedora/Wayland/Vulkan acceptance, no GPU or display-dependent claim, and no
  `drl-desktop` crate, native window, or input/audio backend work.
- No change to gameplay semantics, replay wire or RNG semantics, rendering
  equations, persistence codec, browser contracts, or any `#[wasm_bindgen]`
  export signature.
- No duplicate full repository suite inside the Fedora job; it carries targeted
  evidence only.
- No removal or weakening of macOS coverage, and no attempt to make Ubuntu
  stand in for Fedora-specific package-set evidence.

### 2.7 Evidence boundary

This slice proves that the repository contract suite is runnable on Linux and that
a clean Fedora 43 userland, provisioned with its toolchain packages, builds the
platform-adjacent crates and passes the `drl-core`/`drl-protocol` contracts without
an additional project-specific native-library package. It does not
prove any GPU, Wayland, or display behavior: the Fedora container has no `/dev/dri`
and no `libvulkan`, so those remain `NOT_RUN` here and belong to the
Fedora/Wayland/Vulkan acceptance slice. The Linux job runs the same suite as the
macOS job on Ubuntu; only the Fedora job speaks to the Fedora package set. The
Fedora job checks three crates natively and tests two crates, not the workspace.

### 2.8 Delivery evidence

Evidence is bound to a named revision; a later commit does not inherit an earlier
commit's checks.

- **Implementation:** branch `codex/linux-ci-checks` from `main` at `85e50c4`
  (`0.2.341`, the PR #457 merge checkpoint). Merge into `main` is pending.
- **Fedora container proof, Fedora 43 host (podman, `fedora:43`):**
  `sh scripts/check-fedora-dev.sh` exits `0` (`/tmp/fedora-dev-probe.log` for the
  script-level run and `/tmp/fedora-ci-shape.log` for the exact workflow
  invocation). Toolchain: `rustc 1.98.0 (Fedora 1.98.0-1.fc43)` from `dnf`; the
  workflow provisions `git`, `which`, `rust`, `cargo`, `clippy`, and `rustfmt`, with
  no additional project-specific native-library package needed. Capability probe:
  `dri=absent`,
  `vulkan=library-absent`, `wayland_session=absent`,
  `gpu_and_wayland_acceptance=NOT_RUN`.
- **Local host gates:** `sh scripts/check-repository.sh` exits `0` on the Fedora 43
  host against `c308167` plus the document-only edits carried by this ledger's
  commit (`/tmp/loop2-repo.log` and the re-run recorded in the PR).
- **Hosted observation:** on `c308167`, `Repository checks` (macOS),
  `Repository checks (Linux)`, `Fedora 43 development host`, `WASM browser checks`,
  and `Review policy` all pass. The Linux job log shows 59 `test result: ok.`
  summaries and 1088 passing assertions, so the added coverage really runs the
  workspace suite rather than exiting vacuously. The merging commit's own run is
  still required.
- **Independent review:** pending; this slice claims no verdict until it lands.
- **Hosted checks:** tracked per commit in the PR rather than asserted here.
- **`NOT_RUN`:** GPU, Vulkan, Wayland, and interactive browser acceptance;
  controlled legacy reference captures.

## 3. Enduring invariants

The active slice must preserve:

1. no ambient state, platform APIs, filesystem, browser, or presentation policy
   in `drl-core`;
2. identical declared seed, commands, and semantics produce identical current
   simulation results;
3. incompatible histories fail explicitly before simulation;
4. rejected commands and rejected restores do not partially mutate authoritative
   simulation state;
5. renderers, browser code, MCP, and bots consume fair observations/events and
   do not inspect hidden core state;
6. presentation timing and storage side effects do not advance gameplay;
7. no runtime Lua or generic callback recreation;
8. current-Rust, cross-version, legacy, browser, audiovisual, and performance
   evidence remain separately labeled.
