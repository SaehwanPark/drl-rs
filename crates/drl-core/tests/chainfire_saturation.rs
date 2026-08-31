use drl_core::chainfire::chainfire_profile;
use drl_core::game::Game;
use drl_core::item::Item;
use drl_core::replay::ReplayEngine;
use drl_protocol::{
  Command, CommandError, EntityId, EquipmentSlot, GameEvent, ItemArchetype, ItemId, ItemSpawnKind,
  MonsterSpawnSpec, PlayerSpawnConfig, Position, ReplayLog,
};

type WeaponFactory = fn(ItemId) -> Item;

fn equipped_weapon(seed: u64, make_weapon: WeaponFactory) -> Game {
  let mut game = Game::new(seed, 10, 6, Position::new(2, 2)).unwrap();
  let player_id = game.world().player_id().unwrap();
  let weapon_id = game.world_mut().allocate_item_id();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, make_weapon(weapon_id))
    .unwrap();
  game
}

fn configure_chainfire(game: &mut Game, current_clip: u32) {
  let player_id = game.world().player_id().unwrap();
  let properties = game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap();
  properties.chainfire_level = u8::MAX;
  properties.current_clip = current_clip;
}

fn ranged_events(events: &[GameEvent], player_id: EntityId) -> usize {
  events
    .iter()
    .filter(|event| {
      matches!(
        event,
        GameEvent::AttackResolved {
          attacker_id,
          is_ranged: true,
          ..
        } if *attacker_id == player_id
      )
    })
    .count()
}

#[test]
fn every_chainfire_family_accepts_and_preserves_saturated_state() {
  let cases: &[(ItemArchetype, WeaponFactory, u32, u32)] = &[
    (ItemArchetype::Bfg10k, Item::bfg10k, 7, 35),
    (ItemArchetype::Chaingun, Item::chaingun, 6, 6),
    (ItemArchetype::Minigun, Item::minigun, 12, 12),
    (ItemArchetype::PlasmaRifle, Item::plasma_rifle, 9, 9),
    (ItemArchetype::LaserRifle, Item::laser_rifle, 7, 7),
    (
      ItemArchetype::NuclearPlasmaRifle,
      Item::nuclear_plasma_rifle,
      9,
      9,
    ),
  ];

  for (index, &(archetype, make_weapon, expected_projectiles, expected_cost)) in
    cases.iter().enumerate()
  {
    let target = Position::new(5, 2);
    let mut game = equipped_weapon(3_400 + index as u64, make_weapon);
    let target_id = game
      .world_mut()
      .spawn_monster(target, "Static Target", 100_000, 0, (1, 7))
      .unwrap();
    let player_id = game.world().player_id().unwrap();
    configure_chainfire(&mut game, expected_cost);

    let profile = chainfire_profile(archetype, u8::MAX).expect("registered chainfire family");
    assert_eq!(profile.projectile_count(), expected_projectiles);
    assert_eq!(profile.ammo_cost(), expected_cost);

    let events = game
      .step(Command::AttackRangedChainfire(target))
      .unwrap_or_else(|error| panic!("saturated {archetype:?} burst rejected: {error}"));
    assert_eq!(
      ranged_events(&events, player_id),
      expected_projectiles as usize
    );
    assert!(events.iter().any(|event| matches!(
      event,
      GameEvent::AttackResolved {
        attacker_id,
        target_id: event_target,
        is_ranged: true,
        ..
      } if *attacker_id == player_id && *event_target == target_id
    )));

    let properties = game
      .world()
      .player()
      .unwrap()
      .equipment()
      .weapon()
      .unwrap()
      .weapon_properties()
      .unwrap();
    assert_eq!(properties.current_clip, 0);
    assert_eq!(properties.chainfire_level, u8::MAX);
    assert_eq!(
      game
        .world()
        .player()
        .unwrap()
        .equipment()
        .weapon()
        .unwrap()
        .archetype(),
      archetype
    );
  }
}

#[test]
fn every_chainfire_family_rejects_saturated_under_supply_atomically() {
  let cases: &[(ItemArchetype, WeaponFactory, u32)] = &[
    (ItemArchetype::Bfg10k, Item::bfg10k, 35),
    (ItemArchetype::Chaingun, Item::chaingun, 6),
    (ItemArchetype::Minigun, Item::minigun, 12),
    (ItemArchetype::PlasmaRifle, Item::plasma_rifle, 9),
    (ItemArchetype::LaserRifle, Item::laser_rifle, 7),
    (
      ItemArchetype::NuclearPlasmaRifle,
      Item::nuclear_plasma_rifle,
      9,
    ),
  ];

  for (index, &(archetype, make_weapon, expected_cost)) in cases.iter().enumerate() {
    let target = Position::new(5, 2);
    let mut game = equipped_weapon(3_500 + index as u64, make_weapon);
    game
      .world_mut()
      .spawn_monster(target, "Static Target", 100_000, 0, (1, 7))
      .unwrap();
    configure_chainfire(&mut game, expected_cost - 1);
    let before = game.clone();

    assert_eq!(
      game
        .step(Command::AttackRangedChainfire(target))
        .unwrap_err(),
      CommandError::NoAmmoInClip,
      "saturated {archetype:?} under-supply must reject as a whole burst"
    );
    assert_eq!(
      game, before,
      "saturated {archetype:?} rejection mutated state"
    );
  }
}

#[test]
fn saturated_chainfire_replay_is_repeatable() {
  let player_position = Position::new(2, 2);
  let target = Position::new(5, 2);
  let mut replay =
    ReplayLog::new(3_600, 10, 6, player_position).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: vec![ItemSpawnKind::Ammo9mm(4_000)],
      equipped_weapon: Some(ItemSpawnKind::Minigun),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  replay.record_monster(MonsterSpawnSpec::new(
    target,
    "Static Target",
    1_000_000,
    0,
    (1, 6),
  ));

  // Reload after every burst so the replay reaches the exact level-255
  // boundary without relying on a hidden state injection.
  for _ in 0..255 {
    replay.record_command(Command::AttackRangedChainfire(target));
    replay.record_command(Command::Reload);
  }
  replay.record_command(Command::AttackRangedChainfire(target));

  let (first, first_events) =
    ReplayEngine::run(&replay).expect("saturated chainfire replay should execute");
  let (second, second_events) =
    ReplayEngine::run(&replay).expect("saturated chainfire replay should repeat");
  assert_eq!(first, second);
  assert_eq!(first_events, second_events);
  assert!(ReplayEngine::verify_determinism(&replay).expect("saturated replay determinism"));

  let properties = first
    .world()
    .player()
    .unwrap()
    .equipment()
    .weapon()
    .unwrap()
    .weapon_properties()
    .unwrap();
  assert_eq!(properties.chainfire_level, u8::MAX);
  assert_eq!(properties.current_clip, 188);
}
