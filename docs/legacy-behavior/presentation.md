# Legacy presentation evidence

Status: evidence catalogued; reference capture execution is `NOT_RUN` on the
current arm64 macOS host.

The source revision used for this catalog is
`17d9be1204751899b2d69d8d3a2dde247bd0cc5c`. The legacy renderer describes a
layered 2D sprite map in `src/drlspritemap.pas`, animation timing in
`src/drlanimation.pas`, particle effects in `src/drlparticles.pas`, and color
post-processing LUTs in `bin/data/drl/graphics/`. These are evidence of
capabilities, not an architecture to copy into `drl-core`.

| Capability | Evidence | DRL-Rust target | Status |
| --- | --- | --- | --- |
| Base, emissive, mask, and shadow sheets | `drlspritemap.pas`; imported atlas | `drl-render` layer descriptors and M8 GPU compositing | observed |
| Fog, explored memory, and visibility tint | `drlgfxio.pas`; `drlspritemap.pas` | `RenderScene` visibility flags and M8 lighting | observed/inferred |
| Target overlays and hit/knockback effects | `drlspritemap.pas`; `drlparticles.pas` | event-driven bounded effects | observed |
| Particle burst range sampling | `fpcvalkyrie/src/vrltools.pas`; `drlparticles.pas` | caller-owned renderer range math | observed |
| Particle decal cell mapping | `drlparticles.pas` | caller-owned renderer cell math | observed |
| Particle decal placement | `drlparticles.pas` | caller-owned renderer cell/pixel math | observed |
| Particle decal eligibility | `drlparticles.pas` | caller-owned renderer eligibility gate | observed |
| Particle decal insertion | `drlparticles.pas` | caller-owned renderer insertion request | observed |
| Animation sequencing | `drlanimation.pas` | deterministic presentation timeline | observed |
| HUD, inventory, low-life treatment | `drlgfxio.pas`; `low_life_glow.png` | semantic DOM HUD plus pixel effects | observed/inferred |
| LUT color grading | `lut_*.png` | optional M8 post-process | observed |

M7 wires the full imported graphics atlas to every currently implemented tile,
actor, and item archetype through stable semantic descriptors and presents a
deterministic geometry fallback. Exact rectangle coordinates, texture-layer
compositing, and timing tolerances remain a capture-backed M8 task; the current
descriptor rectangles are deliberately conservative placeholders until the
legacy atlas is measured in the controlled capture environment.
