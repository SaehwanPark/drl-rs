//! Immutable scalar metadata for verified legacy special levels.
//!
//! This catalog is descriptive only. It does not select, generate, or execute
//! a legacy level, and dynamic Lua fields remain outside the Rust boundary.

/// Scalar metadata retained for one verified legacy special level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpecialLevelDefinition {
  pub id: &'static str,
  pub name: &'static str,
  pub legacy_depth: Option<u32>,
  pub entry: Option<&'static str>,
  pub welcome: Option<&'static str>,
}

/// Sorted special-level metadata projected from the pinned evidence index.
pub const SPECIAL_LEVEL_DEFINITIONS: [SpecialLevelDefinition; 26] = [
  SpecialLevelDefinition {
    id: "abyssal_plains",
    name: "Abyssal Plains",
    legacy_depth: Some(12),
    entry: Some("On @1 he romped upon the Abyssal Plains."),
    welcome: Some("You enter the Abyssal Plains. Well isn't this... just... dandy."),
  },
  SpecialLevelDefinition {
    id: "central_processing",
    name: "Central Processing",
    legacy_depth: Some(4),
    entry: Some("On @1 he trekked through Central Processing."),
    welcome: Some(
      "You enter Central Processing. You shudder, thinking about the evil mastermind who planned this.",
    ),
  },
  SpecialLevelDefinition {
    id: "city_of_skulls",
    name: "City of Skulls",
    legacy_depth: Some(12),
    entry: Some("On @1 he found the City of Skulls."),
    welcome: Some("You enter a city made out of bones. You sense a certain tension."),
  },
  SpecialLevelDefinition {
    id: "containment_area",
    name: "Containment Area",
    legacy_depth: Some(11),
    entry: Some("On @1 he arrived at the Containment Area."),
    welcome: Some("You enter the Containment Area. You feel something is hidden behind this wall."),
  },
  SpecialLevelDefinition {
    id: "deimos_lab",
    name: "Deimos Lab",
    legacy_depth: Some(9),
    entry: Some("On @1 he entered Deimos Lab."),
    welcome: Some("You arrive at the Deimos Lab entry area."),
  },
  SpecialLevelDefinition {
    id: "dis",
    name: "Dis",
    legacy_depth: None,
    entry: None,
    welcome: Some("You enter the damned city of Dis..."),
  },
  SpecialLevelDefinition {
    id: "halls_of_carnage",
    name: "Halls of Carnage",
    legacy_depth: Some(14),
    entry: Some("On @1 he ventured into the Halls of Carnage."),
    welcome: Some("You enter the Halls of Carnage. You feel you need to run!"),
  },
  SpecialLevelDefinition {
    id: "hell_fortress",
    name: "Hell Fortress",
    legacy_depth: None,
    entry: None,
    welcome: Some("This is it. This is the lair of all evil! What will you meet here?"),
  },
  SpecialLevelDefinition {
    id: "hellgate",
    name: "Phobos Anomaly",
    legacy_depth: None,
    entry: None,
    welcome: Some("You arrive at the Phobos Anomaly."),
  },
  SpecialLevelDefinition {
    id: "hells_arena",
    name: "Hell's Arena",
    legacy_depth: Some(2),
    entry: Some("On @1 he entered Hell's Arena."),
    welcome: Some("You enter Hell's Arena"),
  },
  SpecialLevelDefinition {
    id: "hells_armory",
    name: "Hell's Armory",
    legacy_depth: Some(9),
    entry: Some("On @1 he entered Hell's Armory."),
    welcome: Some("You enter Hell's Armory."),
  },
  SpecialLevelDefinition {
    id: "house_of_pain",
    name: "House of Pain",
    legacy_depth: Some(17),
    entry: Some("On @1 he trespassed on the House of Pain."),
    welcome: Some("You enter the House of Pain."),
  },
  SpecialLevelDefinition {
    id: "intro",
    name: "Phobos Base Entry",
    legacy_depth: None,
    entry: None,
    welcome: None,
  },
  SpecialLevelDefinition {
    id: "limbo",
    name: "Limbo",
    legacy_depth: Some(20),
    entry: Some("On @1 he was foolish enough to enter Limbo!"),
    welcome: Some("You arrive at Limbo."),
  },
  SpecialLevelDefinition {
    id: "military_base",
    name: "Military Base",
    legacy_depth: Some(7),
    entry: Some("On @1 he marched into the Military Base."),
    welcome: Some("You enter the Military Base. Arriving here again sure takes you back!"),
  },
  SpecialLevelDefinition {
    id: "mt_erebus",
    name: "Mt. Erebus",
    legacy_depth: Some(22),
    entry: Some("On @1 he arrived at Mt. Erebus."),
    welcome: Some("You arrive at Mt. Erebus. You shiver before the mountain of eternal fire!"),
  },
  SpecialLevelDefinition {
    id: "phobos_lab",
    name: "Phobos Lab",
    legacy_depth: Some(7),
    entry: Some("On @1 he sneaked into the Phobos Lab."),
    welcome: Some("You arrive at the Phobos Lab. You are overcome by the feeling of nostalgia!"),
  },
  SpecialLevelDefinition {
    id: "spiders_lair",
    name: "Spider's Lair",
    legacy_depth: Some(14),
    entry: Some("On @1 he ventured into the Spider's Lair."),
    welcome: Some(
      "You descend into the Spider's Lair. Mechanical clicks everywhere! Oh my god it's full of spiders!",
    ),
  },
  SpecialLevelDefinition {
    id: "the_chained_court",
    name: "The Chained Court",
    legacy_depth: Some(5),
    entry: Some("On @1 he stormed the Chained Court."),
    welcome: Some("Welcome to the Chained Court..."),
  },
  SpecialLevelDefinition {
    id: "the_lava_pits",
    name: "The Lava Pits",
    legacy_depth: Some(22),
    entry: Some("On @1 he entered the Lava Pits."),
    welcome: Some("You descend into the Lava Pits. Dammit, it's hot in here!"),
  },
  SpecialLevelDefinition {
    id: "the_mortuary",
    name: "The Mortuary",
    legacy_depth: Some(20),
    entry: Some("On @1 he was foolish enough to enter the Mortuary!"),
    welcome: Some("You enter the Mortuary."),
  },
  SpecialLevelDefinition {
    id: "the_vaults",
    name: "The Vaults",
    legacy_depth: Some(17),
    entry: Some("On @1 he entered the Vaults."),
    welcome: Some("You enter the Vaults. There's a presence here..."),
  },
  SpecialLevelDefinition {
    id: "the_wall",
    name: "The Wall",
    legacy_depth: Some(11),
    entry: Some("On @1 he witnessed the Wall."),
    welcome: Some("You arrive at the Wall. You feel uneasy."),
  },
  SpecialLevelDefinition {
    id: "tower_of_babel",
    name: "Tower of Babel",
    legacy_depth: None,
    entry: None,
    welcome: Some(
      "You enter a big arena. There's blood everywhere. You hear heavy mechanical footsteps...",
    ),
  },
  SpecialLevelDefinition {
    id: "toxin_refinery",
    name: "Toxin Refinery",
    legacy_depth: Some(4),
    entry: Some("On @1 he waded into the Toxin Refinery."),
    welcome: Some("The stench of toxins chokes you briefly."),
  },
  SpecialLevelDefinition {
    id: "unholy_cathedral",
    name: "Unholy Cathedral",
    legacy_depth: Some(19),
    entry: Some("On @1 he invaded the Unholy Cathedral!"),
    welcome: Some("You arrive at the Unholy Cathedral. You feel something sinister in the air."),
  },
];

/// Finds one catalog entry by its stable legacy ID.
#[must_use]
pub fn by_id(id: &str) -> Option<&'static SpecialLevelDefinition> {
  SPECIAL_LEVEL_DEFINITIONS
    .binary_search_by(|definition| definition.id.cmp(id))
    .ok()
    .map(|index| &SPECIAL_LEVEL_DEFINITIONS[index])
}

/// Iterates catalog entries that recorded the given legacy depth.
pub fn at_legacy_depth(depth: u32) -> impl Iterator<Item = &'static SpecialLevelDefinition> {
  SPECIAL_LEVEL_DEFINITIONS
    .iter()
    .filter(move |definition| definition.legacy_depth == Some(depth))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn catalog_is_sorted_and_unique() {
    assert_eq!(SPECIAL_LEVEL_DEFINITIONS.len(), 26);
    for pair in SPECIAL_LEVEL_DEFINITIONS.windows(2) {
      assert!(pair[0].id < pair[1].id);
    }
  }

  #[test]
  fn lookup_preserves_scalar_evidence_and_gaps() {
    let arena = by_id("hells_arena").expect("Hell's Arena metadata");
    assert_eq!(arena.legacy_depth, Some(2));
    assert_eq!(arena.entry, Some("On @1 he entered Hell's Arena."));
    assert_eq!(arena.welcome, Some("You enter Hell's Arena"));

    let intro = by_id("intro").expect("intro metadata");
    assert_eq!(intro.name, "Phobos Base Entry");
    assert_eq!(intro.legacy_depth, None);
    assert_eq!(intro.entry, None);
    assert_eq!(intro.welcome, None);
    assert!(by_id("the_asmos_den").is_none());
    assert!(by_id("unknown").is_none());
  }

  #[test]
  fn depth_lookup_is_deterministic_and_non_selecting() {
    let depth_four: Vec<_> = at_legacy_depth(4).map(|definition| definition.id).collect();
    assert_eq!(depth_four, ["central_processing", "toxin_refinery"]);
    let depth_seven: Vec<_> = at_legacy_depth(7).map(|definition| definition.id).collect();
    assert_eq!(depth_seven, ["military_base", "phobos_lab"]);
    assert!(at_legacy_depth(999).next().is_none());
  }
}
