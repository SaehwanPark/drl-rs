# ADR 0006 — MCP Semantic Interface Strategy

**Status:** Accepted

**Date:** 2026-08-18

---

## Context

DRL-Rust is designed from the start to be operable by AI agents, automated
test bots, and external tools — not just human players at a keyboard. This
requires a well-defined machine-readable interface to the game simulation.

Two design paths were considered:

1. **Raw simulation API exposure** — expose Rust types and function calls
   directly over some transport (e.g., gRPC, JSON-over-pipes). Simple to
   implement, but requires every client to understand simulation internals.

2. **Semantic tool interface** — expose a set of named, documented tools with
   typed parameters and structured responses that mirror player-visible
   concepts. Clients interact in terms of game actions ("move north", "reload",
   "pick up item") rather than Rust function calls.

Option 2 aligns with the Model Context Protocol (MCP), which provides a
standard JSON-RPC 2.0 framing for tool-based agent interfaces.

---

## Decision

DRL-Rust implements an MCP server (`drl-mcp`) as the primary machine-operable
interface for AI agents, automated test players, and integration testing.

The key design principles are:

### 1. MCP is not a simulation bypass

All MCP tools submit `Command` values through the standard `Game::step` API.
MCP agents are subject to the same command validation, information boundaries,
and event semantics as human players. There is no "privileged MCP path" to
game state.

### 2. Observations follow player information boundaries

By default, MCP game state observations are filtered through the same
`PlayerObservation` pipeline used for all other clients. Entities outside the
player's field of view are hidden. Unexplored tiles are masked. The MCP
server cannot return information the player could not see, unless explicitly
operating in developer mode.

### 3. Developer mode is explicitly gated

`game_get_dev_state` and other omniscient access paths require an explicit
`dev_mode: true` parameter. Developer mode is designed for testing and
debugging, not for agent training or fairness-sensitive evaluation.

### 4. Replay determinism is preserved

MCP-driven sessions are recorded as `ReplayLog` entries. A session driven
entirely through MCP tools is reproducible by re-executing the same command
stream from the same seed.

### 5. Transport is stdio JSON-RPC 2.0

The MCP server uses stdin/stdout JSON-RPC 2.0 as its transport. This makes
it trivially hostable as a subprocess by any client without network
configuration, port allocation, or authentication infrastructure.

### 6. No arbitrary filesystem or shell access

The MCP server does not expose tools for filesystem traversal, shell
execution, or any operation outside the game simulation. Tool arguments are
validated; unexpected fields are rejected.

### 7. Semantic, not structural, tool design

Tool names and parameters use player-facing vocabulary ("move north", "reload
weapon", "pick up item") rather than Rust struct field paths. This makes the
interface stable across internal refactoring.

---

## Consequences

- `drl-mcp` depends on `drl-core` and `drl-protocol` but not on any
  platform, rendering, or audio crate.
- AI agent developers interact through documented, stable JSON-RPC tool
  signatures rather than compiled Rust code.
- Test suites can drive complete game episodes over MCP and verify replay
  reproducibility end-to-end.
- The MCP interface is explicitly excluded from `drl-core`'s dependency graph,
  preserving the simulation kernel's purity.
- Future expansion of the tool set (new commands, new resources) follows the
  standard MCP tool-definition mechanism without modifying the simulation API.
