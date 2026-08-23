//! Immutable Rust-owned definitions for the current item spawn families.

use drl_protocol::{ActionCost, AmmoType, ItemArchetype, ItemSpawnKind};

/// Static payload needed to construct one known item family.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemDefinitionKind {
  /// A weapon with its immutable combat properties.
  Weapon {
    is_ranged: bool,
    ammo_type: Option<AmmoType>,
    clip_capacity: u32,
    damage: (u32, u32),
    range: u32,
    accuracy: i32,
    knockback: u32,
    fire_cost: ActionCost,
    reload_cost: ActionCost,
  },
  /// A stackable ammunition family; the instance count remains caller-owned.
  Ammo {
    ammo_type: AmmoType,
    max_stack: u32,
    /// Source-backed amount for a canonical loose-ammo pickup, when known.
    initial_amount: Option<u32>,
  },
  /// A prepared-slot ammunition pack; consumption remains a future slice.
  AmmoPack {
    ammo_type: AmmoType,
    amount: u32,
    max_amount: u32,
  },
  /// A medical consumable with a fixed healing amount.
  MedPack { heal_amount: u32 },
  /// Wearable armor with its baseline durability.
  Armor {
    protection: u32,
    durability: u32,
    max_durability: u32,
  },
  /// The special phase-shift consumable.
  PhaseDevice,
}

/// Immutable metadata for one current Rust-owned item family.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemDefinition {
  pub archetype: ItemArchetype,
  pub name: &'static str,
  pub description: &'static str,
  pub kind: ItemDefinitionKind,
}

impl ItemDefinition {
  /// Returns the pinned initial amount for a canonical ammo pickup, when
  /// available. Replay and scenario callers still own explicit spawn counts.
  #[must_use]
  pub const fn initial_stack_count(self) -> Option<u32> {
    match self.kind {
      ItemDefinitionKind::Ammo { initial_amount, .. } => initial_amount,
      _ => None,
    }
  }
}

const PISTOL: ItemDefinition = ItemDefinition {
  archetype: ItemArchetype::Pistol,
  name: "Pistol",
  description: "Standard 9mm military sidearm. Reliable and accurate.",
  kind: ItemDefinitionKind::Weapon {
    is_ranged: true,
    ammo_type: Some(AmmoType::Ammo9mm),
    clip_capacity: 10,
    damage: (4, 8),
    range: 8,
    accuracy: 75,
    knockback: 0,
    fire_cost: ActionCost::RANGED_ATTACK,
    reload_cost: ActionCost::STANDARD,
  },
};

const SHOTGUN: ItemDefinition = ItemDefinition {
  archetype: ItemArchetype::Shotgun,
  name: "Shotgun",
  description: "Pump-action 12-gauge shotgun. Devastating at close range.",
  kind: ItemDefinitionKind::Weapon {
    is_ranged: true,
    ammo_type: Some(AmmoType::Shells),
    clip_capacity: 8,
    damage: (8, 16),
    range: 5,
    accuracy: 65,
    knockback: 1,
    fire_cost: ActionCost::RANGED_ATTACK,
    reload_cost: ActionCost::new(1200),
  },
};

const DOUBLE_SHOTGUN: ItemDefinition = ItemDefinition {
  archetype: ItemArchetype::DoubleShotgun,
  name: "Double Shotgun",
  description: "Double barreled shotgun -- the perfect weapon for a desperado.",
  kind: ItemDefinitionKind::Weapon {
    is_ranged: true,
    ammo_type: Some(AmmoType::Shells),
    clip_capacity: 2,
    damage: (9, 27),
    range: 8,
    accuracy: 65,
    knockback: 1,
    fire_cost: ActionCost::RANGED_ATTACK,
    reload_cost: ActionCost::STANDARD,
  },
};

const COMBAT_SHOTGUN: ItemDefinition = ItemDefinition {
  archetype: ItemArchetype::CombatShotgun,
  name: "Combat Shotgun",
  description: "Nothing beats the sound of pumping a combat shotgun.",
  kind: ItemDefinitionKind::Weapon {
    is_ranged: true,
    ammo_type: Some(AmmoType::Shells),
    clip_capacity: 5,
    damage: (7, 21),
    range: 15,
    accuracy: 65,
    knockback: 1,
    fire_cost: ActionCost::RANGED_ATTACK,
    reload_cost: ActionCost::STANDARD,
  },
};

const BLASTER: ItemDefinition = ItemDefinition {
  archetype: ItemArchetype::Blaster,
  name: "Blaster",
  description: "This is the standard issue rechargeable energy side-arm. Cool!",
  kind: ItemDefinitionKind::Weapon {
    is_ranged: true,
    ammo_type: Some(AmmoType::Cell),
    clip_capacity: 10,
    damage: (2, 8),
    range: 8,
    accuracy: 70,
    knockback: 0,
    fire_cost: ActionCost::RANGED_ATTACK,
    reload_cost: ActionCost::STANDARD,
  },
};

const LASER_RIFLE: ItemDefinition = ItemDefinition {
  archetype: ItemArchetype::LaserRifle,
  name: "Laser Rifle",
  description: "With no recoil and pinpoint accuracy, it takes a world-class moron to miss while using a laser rifle.",
  kind: ItemDefinitionKind::Weapon {
    is_ranged: true,
    ammo_type: Some(AmmoType::Cell),
    clip_capacity: 40,
    damage: (1, 7),
    range: 8,
    accuracy: 85,
    knockback: 0,
    fire_cost: ActionCost::RANGED_ATTACK,
    reload_cost: ActionCost::STANDARD,
  },
};

const MISSILE_LAUNCHER: ItemDefinition = ItemDefinition {
  archetype: ItemArchetype::MissileLauncher,
  name: "Missile Launcher",
  description: "The definitive upgrade to the rocket launcher.",
  kind: ItemDefinitionKind::Weapon {
    is_ranged: true,
    ammo_type: Some(AmmoType::Rocket),
    clip_capacity: 4,
    damage: (6, 36),
    range: 8,
    accuracy: 75,
    knockback: 0,
    fire_cost: ActionCost::RANGED_ATTACK,
    reload_cost: ActionCost::STANDARD,
  },
};

const NUCLEAR_PLASMA_RIFLE: ItemDefinition = ItemDefinition {
  archetype: ItemArchetype::NuclearPlasmaRifle,
  name: "Nuclear Plasma Rifle",
  description: "A self-charging plasma rifle -- too bad it can't be manually reloaded.",
  kind: ItemDefinitionKind::Weapon {
    is_ranged: true,
    ammo_type: Some(AmmoType::Cell),
    clip_capacity: 24,
    damage: (1, 7),
    range: 8,
    accuracy: 70,
    knockback: 0,
    fire_cost: ActionCost::RANGED_ATTACK,
    reload_cost: ActionCost::STANDARD,
  },
};

const NUCLEAR_BFG_9000: ItemDefinition = ItemDefinition {
  archetype: ItemArchetype::NuclearBfg9000,
  name: "Nuclear BFG 9000",
  description: "A self-charging BFG9000! How much more lucky can you get?",
  kind: ItemDefinitionKind::Weapon {
    is_ranged: true,
    ammo_type: Some(AmmoType::Cell),
    clip_capacity: 40,
    damage: (8, 48),
    range: 8,
    accuracy: 70,
    knockback: 0,
    fire_cost: ActionCost::RANGED_ATTACK,
    reload_cost: ActionCost::STANDARD,
  },
};

const BFG_10K: ItemDefinition = ItemDefinition {
  archetype: ItemArchetype::Bfg10k,
  name: "BFG 10K",
  description: "The Ultimate Big Fucking Gun. Redefines the word \"wallpaper\".",
  kind: ItemDefinitionKind::Weapon {
    is_ranged: true,
    ammo_type: Some(AmmoType::Cell),
    clip_capacity: 50,
    damage: (6, 24),
    range: 8,
    accuracy: 70,
    knockback: 0,
    fire_cost: ActionCost::RANGED_ATTACK,
    reload_cost: ActionCost::STANDARD,
  },
};

const MEGA_BUSTER: ItemDefinition = ItemDefinition {
  archetype: ItemArchetype::MegaBuster,
  name: "Mega Buster",
  description: "You suddenly wish to slaughter the forces of Hell to 8-bit chiptune music.",
  kind: ItemDefinitionKind::Weapon {
    is_ranged: true,
    ammo_type: Some(AmmoType::Ammo9mm),
    clip_capacity: 60,
    damage: (1, 8),
    range: 8,
    accuracy: 70,
    knockback: 0,
    fire_cost: ActionCost::RANGED_ATTACK,
    reload_cost: ActionCost::STANDARD,
  },
};

const GRAMMATON_BERETTA: ItemDefinition = ItemDefinition {
  archetype: ItemArchetype::GrammatonBeretta,
  name: "Grammaton Cleric Beretta",
  description: "No. Not without incident.",
  kind: ItemDefinitionKind::Weapon {
    is_ranged: true,
    ammo_type: Some(AmmoType::Ammo9mm),
    clip_capacity: 18,
    damage: (2, 12),
    range: 8,
    accuracy: 80,
    knockback: 0,
    fire_cost: ActionCost::RANGED_ATTACK,
    reload_cost: ActionCost::STANDARD,
  },
};

const FRAG_SHOTGUN: ItemDefinition = ItemDefinition {
  archetype: ItemArchetype::FragShotgun,
  name: "Frag Shotgun",
  description: "Advanced pulverization technology converts bullets into shrapnel.",
  kind: ItemDefinitionKind::Weapon {
    is_ranged: true,
    ammo_type: Some(AmmoType::Ammo9mm),
    clip_capacity: 16,
    damage: (6, 18),
    range: 15,
    accuracy: 65,
    knockback: 1,
    fire_cost: ActionCost::RANGED_ATTACK,
    reload_cost: ActionCost::STANDARD,
  },
};

const REVENANTS_LAUNCHER: ItemDefinition = ItemDefinition {
  archetype: ItemArchetype::RevenantsLauncher,
  name: "Revenant's Launcher",
  description: "Two can play the homing missile game.",
  kind: ItemDefinitionKind::Weapon {
    is_ranged: true,
    ammo_type: Some(AmmoType::Rocket),
    clip_capacity: 1,
    damage: (7, 42),
    range: 8,
    accuracy: 75,
    knockback: 0,
    fire_cost: ActionCost::RANGED_ATTACK,
    reload_cost: ActionCost::STANDARD,
  },
};

const RAILGUN: ItemDefinition = ItemDefinition {
  archetype: ItemArchetype::Railgun,
  name: "Railgun",
  description: "Groovy! Wait 'til they stand in a row, and watch them being impaled.",
  kind: ItemDefinitionKind::Weapon {
    is_ranged: true,
    ammo_type: Some(AmmoType::Cell),
    clip_capacity: 40,
    damage: (8, 64),
    range: 15,
    accuracy: 90,
    knockback: 0,
    fire_cost: ActionCost::RANGED_ATTACK,
    reload_cost: ActionCost::STANDARD,
  },
};

const ACID_SPITTER: ItemDefinition = ItemDefinition {
  archetype: ItemArchetype::AcidSpitter,
  name: "Acid Spitter",
  description: "Woah, looks cool, but how do I reload it?",
  kind: ItemDefinitionKind::Weapon {
    is_ranged: true,
    ammo_type: Some(AmmoType::Rocket),
    clip_capacity: 10,
    damage: (10, 100),
    range: 15,
    accuracy: 70,
    knockback: 0,
    fire_cost: ActionCost::RANGED_ATTACK,
    reload_cost: ActionCost::STANDARD,
  },
};

const COMBAT_PISTOL: ItemDefinition = ItemDefinition {
  archetype: ItemArchetype::CombatPistol,
  name: "Combat Pistol",
  description: "This is the kind of handgun given to your superiors. Doesn't look like they're using it right now...",
  kind: ItemDefinitionKind::Weapon {
    is_ranged: true,
    ammo_type: Some(AmmoType::Ammo9mm),
    clip_capacity: 15,
    damage: (3, 9),
    range: 8,
    accuracy: 75,
    knockback: 0,
    fire_cost: ActionCost::RANGED_ATTACK,
    reload_cost: ActionCost::STANDARD,
  },
};

const ASSAULT_SHOTGUN: ItemDefinition = ItemDefinition {
  archetype: ItemArchetype::AssaultShotgun,
  name: "Assault Shotgun",
  description: "Big, bad and ugly.",
  kind: ItemDefinitionKind::Weapon {
    is_ranged: true,
    ammo_type: Some(AmmoType::Shells),
    clip_capacity: 6,
    damage: (7, 21),
    range: 15,
    accuracy: 65,
    knockback: 1,
    fire_cost: ActionCost::RANGED_ATTACK,
    reload_cost: ActionCost::STANDARD,
  },
};

const PLASMA_SHOTGUN: ItemDefinition = ItemDefinition {
  archetype: ItemArchetype::PlasmaShotgun,
  name: "Plasma Shotgun",
  description: "Plasma shotgun -- the best of two worlds.",
  kind: ItemDefinitionKind::Weapon {
    is_ranged: true,
    ammo_type: Some(AmmoType::Cell),
    clip_capacity: 30,
    damage: (7, 21),
    range: 15,
    accuracy: 65,
    knockback: 1,
    fire_cost: ActionCost::RANGED_ATTACK,
    reload_cost: ActionCost::STANDARD,
  },
};

const JACKHAMMER: ItemDefinition = ItemDefinition {
  archetype: ItemArchetype::Jackhammer,
  name: "Jackhammer",
  description: "The Pancor Corporation Jackhammer is a 12-gauge, gas-operated automatic weapon.",
  kind: ItemDefinitionKind::Weapon {
    is_ranged: true,
    ammo_type: Some(AmmoType::Shells),
    clip_capacity: 10,
    damage: (8, 24),
    range: 15,
    accuracy: 65,
    knockback: 1,
    fire_cost: ActionCost::RANGED_ATTACK,
    reload_cost: ActionCost::STANDARD,
  },
};

const SUPER_SHOTGUN: ItemDefinition = ItemDefinition {
  archetype: ItemArchetype::SuperShotgun,
  name: "Super Shotgun",
  description: "After the first hellish invasion, weapon engineers designed the super shotgun as the world's first firearm designed to kill demons. And boy does it do a good job.",
  kind: ItemDefinitionKind::Weapon {
    is_ranged: true,
    ammo_type: Some(AmmoType::Shells),
    clip_capacity: 2,
    damage: (8, 32),
    range: 15,
    accuracy: 65,
    knockback: 1,
    fire_cost: ActionCost::RANGED_ATTACK,
    reload_cost: ActionCost::STANDARD,
  },
};

const TRISTAR_BLASTER: ItemDefinition = ItemDefinition {
  archetype: ItemArchetype::TristarBlaster,
  name: "Tristar Blaster",
  description: "Now this is a weird weapon.",
  kind: ItemDefinitionKind::Weapon {
    is_ranged: true,
    ammo_type: Some(AmmoType::Cell),
    clip_capacity: 45,
    damage: (4, 24),
    range: 8,
    accuracy: 70,
    knockback: 0,
    fire_cost: ActionCost::RANGED_ATTACK,
    reload_cost: ActionCost::STANDARD,
  },
};

const BUTCHERS_CLEAVER: ItemDefinition = ItemDefinition {
  archetype: ItemArchetype::ButchersCleaver,
  name: "Butcher's Cleaver",
  description: "Now that is a BIG cleaver. Butcher them!",
  kind: ItemDefinitionKind::Weapon {
    is_ranged: false,
    ammo_type: None,
    clip_capacity: 0,
    damage: (5, 30),
    range: 1,
    accuracy: 85,
    knockback: 0,
    fire_cost: ActionCost::MELEE_ATTACK,
    reload_cost: ActionCost::new(0),
  },
};

const MJOLLNIR: ItemDefinition = ItemDefinition {
  archetype: ItemArchetype::Mjollnir,
  name: "Mjollnir",
  description: "Forged by the dwarves Eitri and Brokk, in response to Loki's challenge, Mjollnir is an indestructible war hammer.",
  kind: ItemDefinitionKind::Weapon {
    is_ranged: false,
    ammo_type: None,
    clip_capacity: 0,
    damage: (1, 25),
    range: 5,
    accuracy: 75,
    knockback: 0,
    fire_cost: ActionCost::MELEE_ATTACK,
    reload_cost: ActionCost::new(0),
  },
};

const SUBTLE_KNIFE: ItemDefinition = ItemDefinition {
  archetype: ItemArchetype::SubtleKnife,
  name: "Subtle Knife",
  description: "A weapon that can cut the very fabric of reality. Too bad it's only eight inches long...",
  kind: ItemDefinitionKind::Weapon {
    is_ranged: false,
    ammo_type: None,
    clip_capacity: 0,
    damage: (3, 15),
    range: 1,
    accuracy: 85,
    knockback: 0,
    fire_cost: ActionCost::MELEE_ATTACK,
    reload_cost: ActionCost::new(0),
  },
};

const TRIGUN: ItemDefinition = ItemDefinition {
  archetype: ItemArchetype::Trigun,
  name: "Trigun",
  description: "One of the deadliest weapons ever made. Nyooo >O.o<",
  kind: ItemDefinitionKind::Weapon {
    is_ranged: true,
    ammo_type: Some(AmmoType::Ammo9mm),
    clip_capacity: 6,
    damage: (3, 18),
    range: 8,
    accuracy: 80,
    knockback: 0,
    fire_cost: ActionCost::RANGED_ATTACK,
    reload_cost: ActionCost::STANDARD,
  },
};

const ANTI_FREAK_JACKAL: ItemDefinition = ItemDefinition {
  archetype: ItemArchetype::AntiFreakJackal,
  name: "Anti-Freak Jackal",
  description: "In the name of God, impure souls of the living dead shall be banished into eternal damnation. Amen.",
  kind: ItemDefinitionKind::Weapon {
    is_ranged: true,
    ammo_type: Some(AmmoType::Ammo9mm),
    clip_capacity: 6,
    damage: (5, 15),
    range: 8,
    accuracy: 75,
    knockback: 0,
    fire_cost: ActionCost::RANGED_ATTACK,
    reload_cost: ActionCost::STANDARD,
  },
};

const MINIGUN: ItemDefinition = ItemDefinition {
  archetype: ItemArchetype::Minigun,
  name: "Minigun",
  description: "Spits enough lead into the air to be considered an environmental hazard.",
  kind: ItemDefinitionKind::Weapon {
    is_ranged: true,
    ammo_type: Some(AmmoType::Ammo9mm),
    clip_capacity: 200,
    damage: (1, 6),
    range: 8,
    accuracy: 70,
    knockback: 0,
    fire_cost: ActionCost::RANGED_ATTACK,
    reload_cost: ActionCost::STANDARD,
  },
};

const CHAINGUN: ItemDefinition = ItemDefinition {
  archetype: ItemArchetype::Chaingun,
  name: "Chaingun",
  description: "Chaingun directs heavy firepower into your opponent making him do the chaingun cha-cha.",
  kind: ItemDefinitionKind::Weapon {
    is_ranged: true,
    ammo_type: Some(AmmoType::Ammo9mm),
    clip_capacity: 40,
    damage: (1, 6),
    range: 8,
    accuracy: 70,
    knockback: 0,
    fire_cost: ActionCost::RANGED_ATTACK,
    reload_cost: ActionCost::STANDARD,
  },
};

const PLASMA_RIFLE: ItemDefinition = ItemDefinition {
  archetype: ItemArchetype::PlasmaRifle,
  name: "Plasma Rifle",
  description: "A plasma rifle shoots multiple rounds of plasma energy -- frying some demon butt!",
  kind: ItemDefinitionKind::Weapon {
    is_ranged: true,
    ammo_type: Some(AmmoType::Cell),
    clip_capacity: 6,
    damage: (1, 7),
    range: 8,
    accuracy: 70,
    knockback: 0,
    fire_cost: ActionCost::RANGED_ATTACK,
    reload_cost: ActionCost::STANDARD,
  },
};

const ROCKET_LAUNCHER: ItemDefinition = ItemDefinition {
  archetype: ItemArchetype::RocketLauncher,
  name: "Rocket Launcher",
  description: "The rocket launcher is the most standard way of blowing things up.",
  kind: ItemDefinitionKind::Weapon {
    is_ranged: true,
    ammo_type: Some(AmmoType::Rocket),
    clip_capacity: 1,
    damage: (6, 36),
    range: 8,
    accuracy: 70,
    knockback: 0,
    fire_cost: ActionCost::RANGED_ATTACK,
    reload_cost: ActionCost::STANDARD,
  },
};

const CHAINSAW: ItemDefinition = ItemDefinition {
  archetype: ItemArchetype::Chainsaw,
  name: "Chainsaw",
  description: "Chainsaw -- cuts through flesh like a hot knife through butter.",
  kind: ItemDefinitionKind::Weapon {
    is_ranged: false,
    ammo_type: None,
    clip_capacity: 0,
    damage: (4, 24),
    range: 1,
    accuracy: 85,
    knockback: 0,
    fire_cost: ActionCost::MELEE_ATTACK,
    reload_cost: ActionCost::new(0),
  },
};

const BFG_9000: ItemDefinition = ItemDefinition {
  archetype: ItemArchetype::Bfg9000,
  name: "BFG 9000",
  description: "The Big Fucking Gun. Hell wouldn't be fun without it.",
  kind: ItemDefinitionKind::Weapon {
    is_ranged: true,
    ammo_type: Some(AmmoType::Cell),
    clip_capacity: 100,
    damage: (10, 60),
    range: 8,
    accuracy: 70,
    knockback: 0,
    fire_cost: ActionCost::RANGED_ATTACK,
    reload_cost: ActionCost::STANDARD,
  },
};

const COMBAT_KNIFE: ItemDefinition = ItemDefinition {
  archetype: ItemArchetype::CombatKnife,
  name: "Combat Knife",
  description: "Serrated combat blade for close-quarters fighting.",
  kind: ItemDefinitionKind::Weapon {
    is_ranged: false,
    ammo_type: None,
    clip_capacity: 0,
    damage: (5, 9),
    range: 1,
    accuracy: 85,
    knockback: 0,
    fire_cost: ActionCost::MELEE_ATTACK,
    reload_cost: ActionCost::new(0),
  },
};

const AMMO_9MM: ItemDefinition = ItemDefinition {
  archetype: ItemArchetype::Ammo9mm,
  name: "9mm Ammo",
  description: "Standard magazine rounds for 9mm pistols and submachine guns.",
  kind: ItemDefinitionKind::Ammo {
    ammo_type: AmmoType::Ammo9mm,
    max_stack: 100,
    initial_amount: None,
  },
};

const AMMO_SHELLS: ItemDefinition = ItemDefinition {
  archetype: ItemArchetype::AmmoShells,
  name: "Shotgun Shells",
  description: "Heavy buckshot shells for shotguns.",
  kind: ItemDefinitionKind::Ammo {
    ammo_type: AmmoType::Shells,
    max_stack: 50,
    initial_amount: None,
  },
};

const AMMO_ROCKETS: ItemDefinition = ItemDefinition {
  archetype: ItemArchetype::AmmoRockets,
  name: "Rocket",
  description: "Rockets -- heavy, big and go boom.",
  kind: ItemDefinitionKind::Ammo {
    ammo_type: AmmoType::Rocket,
    max_stack: 10,
    initial_amount: Some(3),
  },
};

const AMMO_CELLS: ItemDefinition = ItemDefinition {
  archetype: ItemArchetype::AmmoCells,
  name: "Power Cell",
  description: "Power cells, the peak of monster frying technology.",
  kind: ItemDefinitionKind::Ammo {
    ammo_type: AmmoType::Cell,
    max_stack: 50,
    initial_amount: Some(20),
  },
};

const AMMO_PACK_ROCKETS: ItemDefinition = ItemDefinition {
  archetype: ItemArchetype::AmmoPackRockets,
  name: "Rocket Box",
  description: "Now this is the REAL 'boombox'! Might be useful in the prepared slot.",
  kind: ItemDefinitionKind::AmmoPack {
    ammo_type: AmmoType::Rocket,
    amount: 25,
    max_amount: 25,
  },
};

const AMMO_PACK_CELLS: ItemDefinition = ItemDefinition {
  archetype: ItemArchetype::AmmoPackCells,
  name: "Power Battery",
  description: "Joules of energetic fun! Might be useful in the prepared slot.",
  kind: ItemDefinitionKind::AmmoPack {
    ammo_type: AmmoType::Cell,
    amount: 120,
    max_amount: 120,
  },
};

const AMMO_PACK_9MM: ItemDefinition = ItemDefinition {
  archetype: ItemArchetype::AmmoPack9mm,
  name: "10mm Ammo Chain",
  description: "That reminds you about action films you've seen long ago. Might be useful in the prepared slot.",
  kind: ItemDefinitionKind::AmmoPack {
    ammo_type: AmmoType::Ammo9mm,
    amount: 250,
    max_amount: 250,
  },
};

const AMMO_PACK_SHELLS: ItemDefinition = ItemDefinition {
  archetype: ItemArchetype::AmmoPackShells,
  name: "Shell Box",
  description: "Packed shells, like sardines! Might be useful in the prepared slot.",
  kind: ItemDefinitionKind::AmmoPack {
    ammo_type: AmmoType::Shells,
    amount: 100,
    max_amount: 100,
  },
};

const SMALL_MEDPACK: ItemDefinition = ItemDefinition {
  archetype: ItemArchetype::SmallMedPack,
  name: "Small MedPack",
  description: "Great to treat a few burns; for major injuries, better find its larger cousin.",
  kind: ItemDefinitionKind::MedPack { heal_amount: 10 },
};

const LARGE_MEDPACK: ItemDefinition = ItemDefinition {
  archetype: ItemArchetype::LargeMedPack,
  name: "Large MedPack",
  description: "Your savior in times of need.",
  kind: ItemDefinitionKind::MedPack { heal_amount: 25 },
};

const GREEN_ARMOR: ItemDefinition = ItemDefinition {
  archetype: ItemArchetype::GreenArmor,
  name: "Green Armor",
  description: "Standard security armor suit absorbing incoming damage.",
  kind: ItemDefinitionKind::Armor {
    protection: 5,
    durability: 100,
    max_durability: 100,
  },
};

const BLUE_ARMOR: ItemDefinition = ItemDefinition {
  archetype: ItemArchetype::BlueArmor,
  name: "Blue Armor",
  description: "Better than green armor, but it might not be enough.",
  kind: ItemDefinitionKind::Armor {
    protection: 2,
    durability: 100,
    max_durability: 100,
  },
};

const RED_ARMOR: ItemDefinition = ItemDefinition {
  archetype: ItemArchetype::RedArmor,
  name: "Red Armor",
  description: "Nice, red and shiny. Look out for it, because if it's gone, you're gone too.",
  kind: ItemDefinitionKind::Armor {
    protection: 4,
    durability: 100,
    max_durability: 100,
  },
};

const PHASE_DEVICE: ItemDefinition = ItemDefinition {
  archetype: ItemArchetype::PhaseDevice,
  name: "Phase Device",
  description: "Emergency phase-shift device. Instantly teleports the user across space.",
  kind: ItemDefinitionKind::PhaseDevice,
};

/// Returns the immutable definition for one current spawn family.
#[must_use]
pub const fn definition_for_spawn_kind(kind: ItemSpawnKind) -> &'static ItemDefinition {
  match kind {
    ItemSpawnKind::Pistol => &PISTOL,
    ItemSpawnKind::Shotgun => &SHOTGUN,
    ItemSpawnKind::DoubleShotgun => &DOUBLE_SHOTGUN,
    ItemSpawnKind::CombatShotgun => &COMBAT_SHOTGUN,
    ItemSpawnKind::Blaster => &BLASTER,
    ItemSpawnKind::LaserRifle => &LASER_RIFLE,
    ItemSpawnKind::MissileLauncher => &MISSILE_LAUNCHER,
    ItemSpawnKind::NuclearPlasmaRifle => &NUCLEAR_PLASMA_RIFLE,
    ItemSpawnKind::NuclearBfg9000 => &NUCLEAR_BFG_9000,
    ItemSpawnKind::Bfg10k => &BFG_10K,
    ItemSpawnKind::MegaBuster => &MEGA_BUSTER,
    ItemSpawnKind::GrammatonBeretta => &GRAMMATON_BERETTA,
    ItemSpawnKind::FragShotgun => &FRAG_SHOTGUN,
    ItemSpawnKind::RevenantsLauncher => &REVENANTS_LAUNCHER,
    ItemSpawnKind::Railgun => &RAILGUN,
    ItemSpawnKind::AcidSpitter => &ACID_SPITTER,
    ItemSpawnKind::CombatPistol => &COMBAT_PISTOL,
    ItemSpawnKind::AssaultShotgun => &ASSAULT_SHOTGUN,
    ItemSpawnKind::PlasmaShotgun => &PLASMA_SHOTGUN,
    ItemSpawnKind::Jackhammer => &JACKHAMMER,
    ItemSpawnKind::SuperShotgun => &SUPER_SHOTGUN,
    ItemSpawnKind::TristarBlaster => &TRISTAR_BLASTER,
    ItemSpawnKind::ButchersCleaver => &BUTCHERS_CLEAVER,
    ItemSpawnKind::Mjollnir => &MJOLLNIR,
    ItemSpawnKind::SubtleKnife => &SUBTLE_KNIFE,
    ItemSpawnKind::Trigun => &TRIGUN,
    ItemSpawnKind::AntiFreakJackal => &ANTI_FREAK_JACKAL,
    ItemSpawnKind::Minigun => &MINIGUN,
    ItemSpawnKind::Chaingun => &CHAINGUN,
    ItemSpawnKind::PlasmaRifle => &PLASMA_RIFLE,
    ItemSpawnKind::RocketLauncher => &ROCKET_LAUNCHER,
    ItemSpawnKind::Bfg9000 => &BFG_9000,
    ItemSpawnKind::Chainsaw => &CHAINSAW,
    ItemSpawnKind::CombatKnife => &COMBAT_KNIFE,
    ItemSpawnKind::Ammo9mm(_) => &AMMO_9MM,
    ItemSpawnKind::AmmoShells(_) => &AMMO_SHELLS,
    ItemSpawnKind::AmmoRockets(_) => &AMMO_ROCKETS,
    ItemSpawnKind::AmmoCells(_) => &AMMO_CELLS,
    ItemSpawnKind::AmmoPackRockets => &AMMO_PACK_ROCKETS,
    ItemSpawnKind::AmmoPackCells => &AMMO_PACK_CELLS,
    ItemSpawnKind::AmmoPack9mm => &AMMO_PACK_9MM,
    ItemSpawnKind::AmmoPackShells => &AMMO_PACK_SHELLS,
    ItemSpawnKind::SmallMedPack => &SMALL_MEDPACK,
    ItemSpawnKind::LargeMedPack => &LARGE_MEDPACK,
    ItemSpawnKind::GreenArmor => &GREEN_ARMOR,
    ItemSpawnKind::BlueArmor => &BLUE_ARMOR,
    ItemSpawnKind::RedArmor => &RED_ARMOR,
    ItemSpawnKind::PhaseDevice => &PHASE_DEVICE,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn definition_table_covers_every_current_spawn_family() {
    let cases = [
      (ItemSpawnKind::Pistol, ItemArchetype::Pistol, "Pistol"),
      (ItemSpawnKind::Shotgun, ItemArchetype::Shotgun, "Shotgun"),
      (
        ItemSpawnKind::DoubleShotgun,
        ItemArchetype::DoubleShotgun,
        "Double Shotgun",
      ),
      (
        ItemSpawnKind::CombatShotgun,
        ItemArchetype::CombatShotgun,
        "Combat Shotgun",
      ),
      (ItemSpawnKind::Blaster, ItemArchetype::Blaster, "Blaster"),
      (
        ItemSpawnKind::LaserRifle,
        ItemArchetype::LaserRifle,
        "Laser Rifle",
      ),
      (
        ItemSpawnKind::MissileLauncher,
        ItemArchetype::MissileLauncher,
        "Missile Launcher",
      ),
      (
        ItemSpawnKind::NuclearPlasmaRifle,
        ItemArchetype::NuclearPlasmaRifle,
        "Nuclear Plasma Rifle",
      ),
      (
        ItemSpawnKind::NuclearBfg9000,
        ItemArchetype::NuclearBfg9000,
        "Nuclear BFG 9000",
      ),
      (ItemSpawnKind::Bfg10k, ItemArchetype::Bfg10k, "BFG 10K"),
      (
        ItemSpawnKind::MegaBuster,
        ItemArchetype::MegaBuster,
        "Mega Buster",
      ),
      (
        ItemSpawnKind::GrammatonBeretta,
        ItemArchetype::GrammatonBeretta,
        "Grammaton Cleric Beretta",
      ),
      (
        ItemSpawnKind::FragShotgun,
        ItemArchetype::FragShotgun,
        "Frag Shotgun",
      ),
      (
        ItemSpawnKind::RevenantsLauncher,
        ItemArchetype::RevenantsLauncher,
        "Revenant's Launcher",
      ),
      (ItemSpawnKind::Railgun, ItemArchetype::Railgun, "Railgun"),
      (
        ItemSpawnKind::AcidSpitter,
        ItemArchetype::AcidSpitter,
        "Acid Spitter",
      ),
      (
        ItemSpawnKind::CombatPistol,
        ItemArchetype::CombatPistol,
        "Combat Pistol",
      ),
      (
        ItemSpawnKind::AssaultShotgun,
        ItemArchetype::AssaultShotgun,
        "Assault Shotgun",
      ),
      (
        ItemSpawnKind::PlasmaShotgun,
        ItemArchetype::PlasmaShotgun,
        "Plasma Shotgun",
      ),
      (
        ItemSpawnKind::Jackhammer,
        ItemArchetype::Jackhammer,
        "Jackhammer",
      ),
      (
        ItemSpawnKind::SuperShotgun,
        ItemArchetype::SuperShotgun,
        "Super Shotgun",
      ),
      (
        ItemSpawnKind::TristarBlaster,
        ItemArchetype::TristarBlaster,
        "Tristar Blaster",
      ),
      (
        ItemSpawnKind::ButchersCleaver,
        ItemArchetype::ButchersCleaver,
        "Butcher's Cleaver",
      ),
      (ItemSpawnKind::Mjollnir, ItemArchetype::Mjollnir, "Mjollnir"),
      (
        ItemSpawnKind::SubtleKnife,
        ItemArchetype::SubtleKnife,
        "Subtle Knife",
      ),
      (ItemSpawnKind::Trigun, ItemArchetype::Trigun, "Trigun"),
      (
        ItemSpawnKind::AntiFreakJackal,
        ItemArchetype::AntiFreakJackal,
        "Anti-Freak Jackal",
      ),
      (ItemSpawnKind::Minigun, ItemArchetype::Minigun, "Minigun"),
      (ItemSpawnKind::Chaingun, ItemArchetype::Chaingun, "Chaingun"),
      (
        ItemSpawnKind::PlasmaRifle,
        ItemArchetype::PlasmaRifle,
        "Plasma Rifle",
      ),
      (
        ItemSpawnKind::RocketLauncher,
        ItemArchetype::RocketLauncher,
        "Rocket Launcher",
      ),
      (ItemSpawnKind::Chainsaw, ItemArchetype::Chainsaw, "Chainsaw"),
      (ItemSpawnKind::Bfg9000, ItemArchetype::Bfg9000, "BFG 9000"),
      (
        ItemSpawnKind::CombatKnife,
        ItemArchetype::CombatKnife,
        "Combat Knife",
      ),
      (
        ItemSpawnKind::Ammo9mm(0),
        ItemArchetype::Ammo9mm,
        "9mm Ammo",
      ),
      (
        ItemSpawnKind::AmmoShells(0),
        ItemArchetype::AmmoShells,
        "Shotgun Shells",
      ),
      (
        ItemSpawnKind::AmmoRockets(0),
        ItemArchetype::AmmoRockets,
        "Rocket",
      ),
      (
        ItemSpawnKind::AmmoCells(0),
        ItemArchetype::AmmoCells,
        "Power Cell",
      ),
      (
        ItemSpawnKind::AmmoPackRockets,
        ItemArchetype::AmmoPackRockets,
        "Rocket Box",
      ),
      (
        ItemSpawnKind::AmmoPackCells,
        ItemArchetype::AmmoPackCells,
        "Power Battery",
      ),
      (
        ItemSpawnKind::AmmoPack9mm,
        ItemArchetype::AmmoPack9mm,
        "10mm Ammo Chain",
      ),
      (
        ItemSpawnKind::AmmoPackShells,
        ItemArchetype::AmmoPackShells,
        "Shell Box",
      ),
      (
        ItemSpawnKind::SmallMedPack,
        ItemArchetype::SmallMedPack,
        "Small MedPack",
      ),
      (
        ItemSpawnKind::LargeMedPack,
        ItemArchetype::LargeMedPack,
        "Large MedPack",
      ),
      (
        ItemSpawnKind::GreenArmor,
        ItemArchetype::GreenArmor,
        "Green Armor",
      ),
      (
        ItemSpawnKind::BlueArmor,
        ItemArchetype::BlueArmor,
        "Blue Armor",
      ),
      (
        ItemSpawnKind::RedArmor,
        ItemArchetype::RedArmor,
        "Red Armor",
      ),
      (
        ItemSpawnKind::PhaseDevice,
        ItemArchetype::PhaseDevice,
        "Phase Device",
      ),
    ];
    for (kind, archetype, name) in cases {
      let definition = definition_for_spawn_kind(kind);
      assert_eq!(definition.archetype, archetype);
      assert_eq!(definition.name, name);
      assert!(!definition.description.is_empty());
    }
  }

  #[test]
  fn definitions_preserve_current_item_properties() {
    let pistol = definition_for_spawn_kind(ItemSpawnKind::Pistol);
    assert_eq!(pistol.archetype, ItemArchetype::Pistol);
    assert_eq!(pistol.name, "Pistol");
    assert_eq!(
      pistol.kind,
      ItemDefinitionKind::Weapon {
        is_ranged: true,
        ammo_type: Some(AmmoType::Ammo9mm),
        clip_capacity: 10,
        damage: (4, 8),
        range: 8,
        accuracy: 75,
        knockback: 0,
        fire_cost: ActionCost::RANGED_ATTACK,
        reload_cost: ActionCost::STANDARD,
      }
    );

    let shotgun = definition_for_spawn_kind(ItemSpawnKind::Shotgun);
    assert_eq!(shotgun.archetype, ItemArchetype::Shotgun);
    assert_eq!(
      shotgun.kind,
      ItemDefinitionKind::Weapon {
        is_ranged: true,
        ammo_type: Some(AmmoType::Shells),
        clip_capacity: 8,
        damage: (8, 16),
        range: 5,
        accuracy: 65,
        knockback: 1,
        fire_cost: ActionCost::RANGED_ATTACK,
        reload_cost: ActionCost::new(1200),
      }
    );

    assert_eq!(
      definition_for_spawn_kind(ItemSpawnKind::DoubleShotgun).kind,
      ItemDefinitionKind::Weapon {
        is_ranged: true,
        ammo_type: Some(AmmoType::Shells),
        clip_capacity: 2,
        damage: (9, 27),
        range: 8,
        accuracy: 65,
        knockback: 1,
        fire_cost: ActionCost::RANGED_ATTACK,
        reload_cost: ActionCost::STANDARD,
      }
    );
    assert_eq!(
      definition_for_spawn_kind(ItemSpawnKind::CombatShotgun).kind,
      ItemDefinitionKind::Weapon {
        is_ranged: true,
        ammo_type: Some(AmmoType::Shells),
        clip_capacity: 5,
        damage: (7, 21),
        range: 15,
        accuracy: 65,
        knockback: 1,
        fire_cost: ActionCost::RANGED_ATTACK,
        reload_cost: ActionCost::STANDARD,
      }
    );

    assert_eq!(
      definition_for_spawn_kind(ItemSpawnKind::Blaster).kind,
      ItemDefinitionKind::Weapon {
        is_ranged: true,
        ammo_type: Some(AmmoType::Cell),
        clip_capacity: 10,
        damage: (2, 8),
        range: 8,
        accuracy: 70,
        knockback: 0,
        fire_cost: ActionCost::RANGED_ATTACK,
        reload_cost: ActionCost::STANDARD,
      }
    );
    assert_eq!(
      definition_for_spawn_kind(ItemSpawnKind::LaserRifle).kind,
      ItemDefinitionKind::Weapon {
        is_ranged: true,
        ammo_type: Some(AmmoType::Cell),
        clip_capacity: 40,
        damage: (1, 7),
        range: 8,
        accuracy: 85,
        knockback: 0,
        fire_cost: ActionCost::RANGED_ATTACK,
        reload_cost: ActionCost::STANDARD,
      }
    );
    assert_eq!(
      definition_for_spawn_kind(ItemSpawnKind::MissileLauncher).kind,
      ItemDefinitionKind::Weapon {
        is_ranged: true,
        ammo_type: Some(AmmoType::Rocket),
        clip_capacity: 4,
        damage: (6, 36),
        range: 8,
        accuracy: 75,
        knockback: 0,
        fire_cost: ActionCost::RANGED_ATTACK,
        reload_cost: ActionCost::STANDARD,
      }
    );
    assert_eq!(
      definition_for_spawn_kind(ItemSpawnKind::NuclearPlasmaRifle).kind,
      ItemDefinitionKind::Weapon {
        is_ranged: true,
        ammo_type: Some(AmmoType::Cell),
        clip_capacity: 24,
        damage: (1, 7),
        range: 8,
        accuracy: 70,
        knockback: 0,
        fire_cost: ActionCost::RANGED_ATTACK,
        reload_cost: ActionCost::STANDARD,
      }
    );
    assert_eq!(
      definition_for_spawn_kind(ItemSpawnKind::NuclearBfg9000).kind,
      ItemDefinitionKind::Weapon {
        is_ranged: true,
        ammo_type: Some(AmmoType::Cell),
        clip_capacity: 40,
        damage: (8, 48),
        range: 8,
        accuracy: 70,
        knockback: 0,
        fire_cost: ActionCost::RANGED_ATTACK,
        reload_cost: ActionCost::STANDARD,
      }
    );
    assert_eq!(
      definition_for_spawn_kind(ItemSpawnKind::Bfg10k).kind,
      ItemDefinitionKind::Weapon {
        is_ranged: true,
        ammo_type: Some(AmmoType::Cell),
        clip_capacity: 50,
        damage: (6, 24),
        range: 8,
        accuracy: 70,
        knockback: 0,
        fire_cost: ActionCost::RANGED_ATTACK,
        reload_cost: ActionCost::STANDARD,
      }
    );
    assert_eq!(
      definition_for_spawn_kind(ItemSpawnKind::MegaBuster).kind,
      ItemDefinitionKind::Weapon {
        is_ranged: true,
        ammo_type: Some(AmmoType::Ammo9mm),
        clip_capacity: 60,
        damage: (1, 8),
        range: 8,
        accuracy: 70,
        knockback: 0,
        fire_cost: ActionCost::RANGED_ATTACK,
        reload_cost: ActionCost::STANDARD,
      }
    );
    assert_eq!(
      definition_for_spawn_kind(ItemSpawnKind::GrammatonBeretta).kind,
      ItemDefinitionKind::Weapon {
        is_ranged: true,
        ammo_type: Some(AmmoType::Ammo9mm),
        clip_capacity: 18,
        damage: (2, 12),
        range: 8,
        accuracy: 80,
        knockback: 0,
        fire_cost: ActionCost::RANGED_ATTACK,
        reload_cost: ActionCost::STANDARD,
      }
    );
    assert_eq!(
      definition_for_spawn_kind(ItemSpawnKind::FragShotgun).kind,
      ItemDefinitionKind::Weapon {
        is_ranged: true,
        ammo_type: Some(AmmoType::Ammo9mm),
        clip_capacity: 16,
        damage: (6, 18),
        range: 15,
        accuracy: 65,
        knockback: 1,
        fire_cost: ActionCost::RANGED_ATTACK,
        reload_cost: ActionCost::STANDARD,
      }
    );
    assert_eq!(
      definition_for_spawn_kind(ItemSpawnKind::RevenantsLauncher).kind,
      ItemDefinitionKind::Weapon {
        is_ranged: true,
        ammo_type: Some(AmmoType::Rocket),
        clip_capacity: 1,
        damage: (7, 42),
        range: 8,
        accuracy: 75,
        knockback: 0,
        fire_cost: ActionCost::RANGED_ATTACK,
        reload_cost: ActionCost::STANDARD,
      }
    );
    assert_eq!(
      definition_for_spawn_kind(ItemSpawnKind::Railgun).kind,
      ItemDefinitionKind::Weapon {
        is_ranged: true,
        ammo_type: Some(AmmoType::Cell),
        clip_capacity: 40,
        damage: (8, 64),
        range: 15,
        accuracy: 90,
        knockback: 0,
        fire_cost: ActionCost::RANGED_ATTACK,
        reload_cost: ActionCost::STANDARD,
      }
    );
    assert_eq!(
      definition_for_spawn_kind(ItemSpawnKind::AcidSpitter).kind,
      ItemDefinitionKind::Weapon {
        is_ranged: true,
        ammo_type: Some(AmmoType::Rocket),
        clip_capacity: 10,
        damage: (10, 100),
        range: 15,
        accuracy: 70,
        knockback: 0,
        fire_cost: ActionCost::RANGED_ATTACK,
        reload_cost: ActionCost::STANDARD,
      }
    );
    assert_eq!(
      definition_for_spawn_kind(ItemSpawnKind::CombatPistol).kind,
      ItemDefinitionKind::Weapon {
        is_ranged: true,
        ammo_type: Some(AmmoType::Ammo9mm),
        clip_capacity: 15,
        damage: (3, 9),
        range: 8,
        accuracy: 75,
        knockback: 0,
        fire_cost: ActionCost::RANGED_ATTACK,
        reload_cost: ActionCost::STANDARD,
      }
    );
    assert_eq!(
      definition_for_spawn_kind(ItemSpawnKind::AssaultShotgun).kind,
      ItemDefinitionKind::Weapon {
        is_ranged: true,
        ammo_type: Some(AmmoType::Shells),
        clip_capacity: 6,
        damage: (7, 21),
        range: 15,
        accuracy: 65,
        knockback: 1,
        fire_cost: ActionCost::RANGED_ATTACK,
        reload_cost: ActionCost::STANDARD,
      }
    );
    assert_eq!(
      definition_for_spawn_kind(ItemSpawnKind::PlasmaShotgun).kind,
      ItemDefinitionKind::Weapon {
        is_ranged: true,
        ammo_type: Some(AmmoType::Cell),
        clip_capacity: 30,
        damage: (7, 21),
        range: 15,
        accuracy: 65,
        knockback: 1,
        fire_cost: ActionCost::RANGED_ATTACK,
        reload_cost: ActionCost::STANDARD,
      }
    );
    assert_eq!(
      definition_for_spawn_kind(ItemSpawnKind::Jackhammer).kind,
      ItemDefinitionKind::Weapon {
        is_ranged: true,
        ammo_type: Some(AmmoType::Shells),
        clip_capacity: 10,
        damage: (8, 24),
        range: 15,
        accuracy: 65,
        knockback: 1,
        fire_cost: ActionCost::RANGED_ATTACK,
        reload_cost: ActionCost::STANDARD,
      }
    );
    assert_eq!(
      definition_for_spawn_kind(ItemSpawnKind::SuperShotgun).kind,
      ItemDefinitionKind::Weapon {
        is_ranged: true,
        ammo_type: Some(AmmoType::Shells),
        clip_capacity: 2,
        damage: (8, 32),
        range: 15,
        accuracy: 65,
        knockback: 1,
        fire_cost: ActionCost::RANGED_ATTACK,
        reload_cost: ActionCost::STANDARD,
      }
    );
    assert_eq!(
      definition_for_spawn_kind(ItemSpawnKind::TristarBlaster).kind,
      ItemDefinitionKind::Weapon {
        is_ranged: true,
        ammo_type: Some(AmmoType::Cell),
        clip_capacity: 45,
        damage: (4, 24),
        range: 8,
        accuracy: 70,
        knockback: 0,
        fire_cost: ActionCost::RANGED_ATTACK,
        reload_cost: ActionCost::STANDARD,
      }
    );
    assert_eq!(
      definition_for_spawn_kind(ItemSpawnKind::ButchersCleaver).kind,
      ItemDefinitionKind::Weapon {
        is_ranged: false,
        ammo_type: None,
        clip_capacity: 0,
        damage: (5, 30),
        range: 1,
        accuracy: 85,
        knockback: 0,
        fire_cost: ActionCost::MELEE_ATTACK,
        reload_cost: ActionCost::new(0),
      }
    );
    assert_eq!(
      definition_for_spawn_kind(ItemSpawnKind::Mjollnir).kind,
      ItemDefinitionKind::Weapon {
        is_ranged: false,
        ammo_type: None,
        clip_capacity: 0,
        damage: (1, 25),
        range: 5,
        accuracy: 75,
        knockback: 0,
        fire_cost: ActionCost::MELEE_ATTACK,
        reload_cost: ActionCost::new(0),
      }
    );
    assert_eq!(
      definition_for_spawn_kind(ItemSpawnKind::SubtleKnife).kind,
      ItemDefinitionKind::Weapon {
        is_ranged: false,
        ammo_type: None,
        clip_capacity: 0,
        damage: (3, 15),
        range: 1,
        accuracy: 85,
        knockback: 0,
        fire_cost: ActionCost::MELEE_ATTACK,
        reload_cost: ActionCost::new(0),
      }
    );
    assert_eq!(
      definition_for_spawn_kind(ItemSpawnKind::Trigun).kind,
      ItemDefinitionKind::Weapon {
        is_ranged: true,
        ammo_type: Some(AmmoType::Ammo9mm),
        clip_capacity: 6,
        damage: (3, 18),
        range: 8,
        accuracy: 80,
        knockback: 0,
        fire_cost: ActionCost::RANGED_ATTACK,
        reload_cost: ActionCost::STANDARD,
      }
    );
    assert_eq!(
      definition_for_spawn_kind(ItemSpawnKind::AntiFreakJackal).kind,
      ItemDefinitionKind::Weapon {
        is_ranged: true,
        ammo_type: Some(AmmoType::Ammo9mm),
        clip_capacity: 6,
        damage: (5, 15),
        range: 8,
        accuracy: 75,
        knockback: 0,
        fire_cost: ActionCost::RANGED_ATTACK,
        reload_cost: ActionCost::STANDARD,
      }
    );
    assert_eq!(
      definition_for_spawn_kind(ItemSpawnKind::Minigun).kind,
      ItemDefinitionKind::Weapon {
        is_ranged: true,
        ammo_type: Some(AmmoType::Ammo9mm),
        clip_capacity: 200,
        damage: (1, 6),
        range: 8,
        accuracy: 70,
        knockback: 0,
        fire_cost: ActionCost::RANGED_ATTACK,
        reload_cost: ActionCost::STANDARD,
      }
    );

    assert_eq!(
      definition_for_spawn_kind(ItemSpawnKind::Chaingun).kind,
      ItemDefinitionKind::Weapon {
        is_ranged: true,
        ammo_type: Some(AmmoType::Ammo9mm),
        clip_capacity: 40,
        damage: (1, 6),
        range: 8,
        accuracy: 70,
        knockback: 0,
        fire_cost: ActionCost::RANGED_ATTACK,
        reload_cost: ActionCost::STANDARD,
      }
    );

    let knife = definition_for_spawn_kind(ItemSpawnKind::CombatKnife);
    assert_eq!(knife.archetype, ItemArchetype::CombatKnife);
    assert_eq!(
      knife.kind,
      ItemDefinitionKind::Weapon {
        is_ranged: false,
        ammo_type: None,
        clip_capacity: 0,
        damage: (5, 9),
        range: 1,
        accuracy: 85,
        knockback: 0,
        fire_cost: ActionCost::MELEE_ATTACK,
        reload_cost: ActionCost::new(0),
      }
    );

    assert_eq!(
      definition_for_spawn_kind(ItemSpawnKind::SmallMedPack).kind,
      ItemDefinitionKind::MedPack { heal_amount: 10 }
    );
    assert_eq!(
      definition_for_spawn_kind(ItemSpawnKind::SmallMedPack).description,
      "Great to treat a few burns; for major injuries, better find its larger cousin."
    );
    assert_eq!(
      definition_for_spawn_kind(ItemSpawnKind::LargeMedPack).kind,
      ItemDefinitionKind::MedPack { heal_amount: 25 }
    );
    assert_eq!(
      definition_for_spawn_kind(ItemSpawnKind::LargeMedPack).description,
      "Your savior in times of need."
    );
    assert_eq!(
      definition_for_spawn_kind(ItemSpawnKind::GreenArmor).kind,
      ItemDefinitionKind::Armor {
        protection: 5,
        durability: 100,
        max_durability: 100,
      }
    );
    assert_eq!(
      definition_for_spawn_kind(ItemSpawnKind::PhaseDevice).kind,
      ItemDefinitionKind::PhaseDevice
    );
    assert_eq!(
      definition_for_spawn_kind(ItemSpawnKind::BlueArmor).kind,
      ItemDefinitionKind::Armor {
        protection: 2,
        durability: 100,
        max_durability: 100,
      }
    );
    assert_eq!(
      definition_for_spawn_kind(ItemSpawnKind::RedArmor).kind,
      ItemDefinitionKind::Armor {
        protection: 4,
        durability: 100,
        max_durability: 100,
      }
    );
    assert_eq!(
      definition_for_spawn_kind(ItemSpawnKind::PlasmaRifle).kind,
      ItemDefinitionKind::Weapon {
        is_ranged: true,
        ammo_type: Some(AmmoType::Cell),
        clip_capacity: 6,
        damage: (1, 7),
        range: 8,
        accuracy: 70,
        knockback: 0,
        fire_cost: ActionCost::RANGED_ATTACK,
        reload_cost: ActionCost::STANDARD,
      }
    );
    assert_eq!(
      definition_for_spawn_kind(ItemSpawnKind::RocketLauncher).kind,
      ItemDefinitionKind::Weapon {
        is_ranged: true,
        ammo_type: Some(AmmoType::Rocket),
        clip_capacity: 1,
        damage: (6, 36),
        range: 8,
        accuracy: 70,
        knockback: 0,
        fire_cost: ActionCost::RANGED_ATTACK,
        reload_cost: ActionCost::STANDARD,
      }
    );

    assert_eq!(
      definition_for_spawn_kind(ItemSpawnKind::Chainsaw).kind,
      ItemDefinitionKind::Weapon {
        is_ranged: false,
        ammo_type: None,
        clip_capacity: 0,
        damage: (4, 24),
        range: 1,
        accuracy: 85,
        knockback: 0,
        fire_cost: ActionCost::MELEE_ATTACK,
        reload_cost: ActionCost::new(0),
      }
    );
    assert_eq!(
      definition_for_spawn_kind(ItemSpawnKind::Bfg9000).kind,
      ItemDefinitionKind::Weapon {
        is_ranged: true,
        ammo_type: Some(AmmoType::Cell),
        clip_capacity: 100,
        damage: (10, 60),
        range: 8,
        accuracy: 70,
        knockback: 0,
        fire_cost: ActionCost::RANGED_ATTACK,
        reload_cost: ActionCost::STANDARD,
      }
    );
  }

  #[test]
  fn ammunition_definitions_keep_family_stack_policies() {
    assert_eq!(
      definition_for_spawn_kind(ItemSpawnKind::Ammo9mm(7)).kind,
      ItemDefinitionKind::Ammo {
        ammo_type: AmmoType::Ammo9mm,
        max_stack: 100,
        initial_amount: None,
      }
    );
    assert_eq!(
      definition_for_spawn_kind(ItemSpawnKind::AmmoShells(7)).kind,
      ItemDefinitionKind::Ammo {
        ammo_type: AmmoType::Shells,
        max_stack: 50,
        initial_amount: None,
      }
    );
    assert_eq!(
      definition_for_spawn_kind(ItemSpawnKind::AmmoRockets(7)).kind,
      ItemDefinitionKind::Ammo {
        ammo_type: AmmoType::Rocket,
        max_stack: 10,
        initial_amount: Some(3),
      }
    );
    assert_eq!(
      definition_for_spawn_kind(ItemSpawnKind::AmmoCells(7)).kind,
      ItemDefinitionKind::Ammo {
        ammo_type: AmmoType::Cell,
        max_stack: 50,
        initial_amount: Some(20),
      }
    );
    assert_eq!(
      definition_for_spawn_kind(ItemSpawnKind::AmmoRockets(99)).initial_stack_count(),
      Some(3)
    );
    assert_eq!(
      definition_for_spawn_kind(ItemSpawnKind::AmmoCells(99)).initial_stack_count(),
      Some(20)
    );
    assert_eq!(
      definition_for_spawn_kind(ItemSpawnKind::Ammo9mm(99)).initial_stack_count(),
      None
    );
    assert_eq!(
      definition_for_spawn_kind(ItemSpawnKind::Pistol).initial_stack_count(),
      None
    );
    assert_eq!(
      definition_for_spawn_kind(ItemSpawnKind::AmmoPackRockets).kind,
      ItemDefinitionKind::AmmoPack {
        ammo_type: AmmoType::Rocket,
        amount: 25,
        max_amount: 25,
      }
    );
    assert_eq!(
      definition_for_spawn_kind(ItemSpawnKind::AmmoPackCells).kind,
      ItemDefinitionKind::AmmoPack {
        ammo_type: AmmoType::Cell,
        amount: 120,
        max_amount: 120,
      }
    );
    assert_eq!(
      definition_for_spawn_kind(ItemSpawnKind::AmmoPack9mm).kind,
      ItemDefinitionKind::AmmoPack {
        ammo_type: AmmoType::Ammo9mm,
        amount: 250,
        max_amount: 250,
      }
    );
    assert_eq!(
      definition_for_spawn_kind(ItemSpawnKind::AmmoPackShells).kind,
      ItemDefinitionKind::AmmoPack {
        ammo_type: AmmoType::Shells,
        amount: 100,
        max_amount: 100,
      }
    );
  }
}
