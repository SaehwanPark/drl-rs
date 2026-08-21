# DRL fidelity matrix

Status: capture execution is `NOT_RUN` on the current arm64 macOS host. The
matrix is the acceptance map; it is not evidence that the named legacy scenes
already match the browser slice.

| Legacy capability/capture | M7 functional target | M8 parity target | Evidence status |
| --- | --- | --- | --- |
| map lighting and fog | visible/explored scene flags and recoverable canvas | measured layer tint, lighting, and LUT tolerances | NOT_RUN |
| targeting and ranged combat | semantic target candidates and command flow | target overlay, muzzle/hit timing, cue timing | NOT_RUN |
| knockback and death | deterministic events and game-over/restart | bounded animation/effect timing and death presentation | NOT_RUN |
| low-health treatment | HP HUD and recoverable presentation | low-life glow/tint and measured threshold | NOT_RUN |
| inventory and HUD | DOM inventory actions and semantic HUD values | sprite layers, typography, layout, and accessibility comparison | NOT_RUN |
| level transition | stairs command and deterministic transition event | transition animation, cleared music/cue timing | NOT_RUN |

Each future row update must retain the legacy revision, executable SHA-256,
configuration, scenario, ordered actions, viewport/DPR, capture-tool versions,
media hashes, rights status, and a stated tolerance. A missing rights or
capture field remains `INCONCLUSIVE`; an unavailable capture environment is
`NOT_RUN`.
