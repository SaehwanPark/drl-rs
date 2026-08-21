# Legacy audio evidence and redistribution gate

`src/drlaudio.pas` and `bin/data/drlhq/audio.lua` establish the legacy cue
categories (movement, weapons, impacts, item use, teleport, transitions, and
music state). The source release README states that sound and music binaries
are downloaded separately; their redistribution rights are not established in
this repository.

The browser M7 slice therefore maps `GameEvent` values to semantic
`drl-audio::AudioCue` values and generates short Web Audio tones after a user
gesture. No legacy audio, music, or font file is bundled. Each future cue may
replace a generated tone only after a provenance record names the source,
license, checksum, and redistribution permission.

Required M8 evidence for audiovisual equivalence:

- cue name, event trigger, duration, and transition timing;
- reference capture manifest with source revision, executable hash,
  configuration, scenario, action stream, and tool versions;
- human comparison plus automated tolerance checks for approved captures.
