# Item Registration Fan-out Inventory

**Status:** Verified inventory for project version `0.2.141`

**Date:** 2026-08-24

This inventory satisfies the Gate C requirement to identify the current manual
fan-out points for an item archetype. It distinguishes routine identity
projections, which should converge on the catalog, from exhaustive mappings and
behavioral tests, where an explicit match is an intentional review boundary.

## Current points of change

| Area | Current location | Responsibility | Status and next action |
| --- | --- | --- | --- |
| Stable identity catalog | `crates/drl-protocol/src/item.rs` (`ItemArchetype`, `ALL`, `stable_name`) | Stable protocol archetype IDs and canonical wire names | Catalog-backed name/display/parsing and uniqueness tests are delivered. Keep the enum and name projection exhaustive. |
| Spawn/replay identity | `crates/drl-protocol/src/replay.rs` (`ItemSpawnKind`) | Typed spawn variants, loose-ammo counts, archetype conversion | `archetype`, `stack_count`, and `from_archetype` are typed projections. Keep count-sensitive and `Unknown` handling explicit. |
| Gameplay family catalog | `crates/drl-protocol/src/replay.rs` (`ItemSpawnKind::ALL`), exposed by `crates/drl-core/src/item_definition.rs` (`CURRENT_ITEM_SPAWN_KINDS`) | Stable representative families and definition-backed structural validation | The stable family list is single-sourced in the protocol spawn contract; core keeps the validation alias and owns balance/behavior. |
| Gameplay definitions/factory | `crates/drl-core/src/item_definition.rs`, `crates/drl-core/src/item.rs` | Immutable balance, stack policy, item construction, and item views | `CURRENT_ITEM_DEFINITIONS` now owns definition lookup and coverage in catalog order; behavior and balance remain core-owned. |
| Replay JSON | `crates/drl-mcp/src/replay_json.rs`, `replay_json_decode.rs` | Wire encoding/decoding of stable item names and optional counts | Uses typed spawn projections and protocol stable-name parsing. No independent name table remains. |
| Atlas descriptors | `crates/drl-assets/src/lib.rs` (`item_sprite`) | Atlas geometry, layers, and evidenced animation metadata | The exhaustive descriptor match remains explicit for compiler coverage. Routine descriptor tests iterate `ItemArchetype::ALL`; geometry still requires a deliberate entry. |
| Render policy | `crates/drl-render/src/lib.rs` | Item-specific colorization and effect policy | Explicit presentation policy, not routine registration. Keep separate until a typed presentation vocabulary is evidenced. |
| Web/UI policy | `crates/drl-web/src/lib.rs` | Inventory markup, browser actions, and item-facing controls | Explicit UI behavior and escaping tests; not an identity registry. |
| Core loot/scenario data | `crates/drl-core/src/loot_definition.rs`, `scenario.rs`, and integration fixtures | Roll payloads, deterministic examples, and scenario setup | These are gameplay evidence and fixtures. They may mention concrete items but are not registration fan-outs. |
| Cross-cutting tests | `crates/drl-core/src/item_definition.rs` tests, replay/MCP tests, asset tests, and integration suites | Preserve invariants, compatibility, and evidence | Routine replay completeness and asset coverage now iterate the catalog; semantic tests and explicit normalized ammo fixture counts remain reviewable. |

## Classification rule

An item change is a routine catalog change when it adds stable identity,
canonical naming, replay representation, structural definition coverage, or
routine descriptor coverage. Those paths should be generated or iterated from
the authoritative catalog where the dependency direction permits it.

An item change is an explicit semantic change when it adds gameplay balance,
stack/count rules, behavior, atlas geometry, rendering effects, browser UI, or
legacy evidence. Those paths remain typed and reviewable even when they require
an exhaustive match or a dedicated test case.

The remaining Gate C work is therefore bounded: identify any routine list not
covered by the catalog, then migrate that list without collapsing semantic
boundaries. The stable protocol identity trio (enum, `ALL`, and wire names) now
comes from one declaration in `0.2.141`; broader content and presentation
catalog convergence remains open. The inventory does not claim that behavior or
presentation parity is complete; those remain Gate D/E work.
