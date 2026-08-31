# Legacy audio evidence and redistribution gate

`src/drlaudio.pas` and `bin/data/drlhq/audio.lua` establish the legacy cue
categories (movement, weapons, impacts, item use, teleport, transitions, and
music state). Sound effects, music tracks, and fonts from the original game are
approved for in-game use, provided they are downloaded separately and not
distributed within the repository or release packages.

The browser edition maps `GameEvent` values to semantic `drl-audio::AudioCue`
values and generates short Web Audio tones by default. Players and developers
can prepare full audio assets locally via `scripts/prepare-legacy-assets.sh`
from official binary distributions or pre-downloaded legacy sources without
tracking them in git or bundling them into static release builds.

Required M8 evidence for audiovisual equivalence:

- cue name, event trigger, duration, and transition timing;
- reference capture manifest with source revision, executable hash,
  configuration, scenario, action stream, and tool versions;
- human comparison plus automated tolerance checks for approved captures.
