//! Fixed-content and chainfire catalog projections at the browser boundary.

use super::*;

#[test]
fn chainfire_cost_projection_accepts_saturated_levels_for_every_family() {
  let cases = [
    (ItemArchetype::Bfg10k, 35),
    (ItemArchetype::Chaingun, 6),
    (ItemArchetype::Minigun, 12),
    (ItemArchetype::PlasmaRifle, 9),
    (ItemArchetype::LaserRifle, 7),
    (ItemArchetype::NuclearPlasmaRifle, 9),
  ];
  for (archetype, expected_cost) in cases {
    assert_eq!(chainfire_ammo_cost(archetype, u8::MAX), Some(expected_cost));
  }
}

#[test]
fn browser_session_accepts_saturated_chainfire_for_every_family() {
  let cases: &[ChainfireWeaponCase] = &[
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
    let mut game =
      Game::new(3_700 + index as u64, 10, 6, Position::new(2, 2)).expect("saturated browser game");
    let player_id = game.world().player_id().expect("player identity");
    let weapon_id = game.world_mut().allocate_item_id();
    game
      .world_mut()
      .get_actor_mut(player_id)
      .expect("browser player")
      .equipment_mut()
      .equip(drl_protocol::EquipmentSlot::Weapon, make_weapon(weapon_id))
      .expect("saturated browser weapon");
    let target_id = game
      .world_mut()
      .spawn_monster(target, "Static Target", 100_000, 0, (1, 7))
      .expect("saturated browser target");
    let properties = game
      .world_mut()
      .get_actor_mut(player_id)
      .expect("browser player")
      .equipment_mut()
      .weapon_mut()
      .expect("browser weapon")
      .weapon_properties_mut()
      .expect("browser weapon properties");
    properties.chainfire_level = u8::MAX;
    properties.current_clip = expected_cost;

    let mut direct = game.clone();
    let mut browser = BrowserSession::from_game(game);
    let command = Command::AttackRangedChainfire(target);
    let expected_events = direct
      .step(command)
      .unwrap_or_else(|error| panic!("direct saturated {archetype:?} burst: {error}"));
    let step = browser
      .submit(command)
      .unwrap_or_else(|error| panic!("browser saturated {archetype:?} burst: {error}"));
    assert_eq!(step.events, expected_events);
    assert_eq!(step.after, direct.observe_player());
    assert_eq!(
      step
        .events
        .iter()
        .filter(|event| matches!(
          event,
          drl_protocol::GameEvent::AttackResolved {
            attacker_id,
            target_id: event_target,
            is_ranged: true,
            ..
          } if *attacker_id == player_id && *event_target == target_id
        ))
        .count(),
      expected_projectiles as usize
    );
    let weapon = step.after.equipped_weapon.expect("browser weapon view");
    assert_eq!(weapon.archetype, archetype);
    assert_eq!(weapon.chainfire_level, u8::MAX);
    assert_eq!(weapon.clip.map(|(loaded, _)| loaded), Some(0));
  }
}
