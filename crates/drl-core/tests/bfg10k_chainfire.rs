use drl_core::game::Game;
use drl_core::item::Item;
use drl_core::replay::ReplayEngine;
use drl_protocol::{
  AttackOutcome, Command, CommandError, EquipmentSlot, GameEvent, ItemSpawnKind, MonsterSpawnSpec,
  PlayerSpawnConfig, Position, ReplayLog,
};

fn equipped_bfg10k(seed: u64) -> Game {
  let mut game = Game::new(seed, 10, 6, Position::new(2, 2)).unwrap();
  let player_id = game.world().player_id().unwrap();
  let weapon_id = game.world_mut().allocate_item_id();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .equip(EquipmentSlot::Weapon, Item::bfg10k(weapon_id))
    .unwrap();
  game
}

fn ranged_events(events: &[GameEvent], player_id: drl_protocol::EntityId) -> usize {
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

fn bfg10k_schedules(events: &[GameEvent], target_id: drl_protocol::EntityId) -> usize {
  events
    .iter()
    .filter(|event| {
      matches!(
        event,
        GameEvent::Bfg10kExplosionScheduled {
          target_id: event_target,
          delay: 25,
          radius: 2,
          knockback: 16,
          ..
        } if *event_target == target_id
      )
    })
    .count()
}

#[test]
fn bfg10k_first_chainfire_emits_four_exact_hits_and_schedules_explosions() {
  let mut game = equipped_bfg10k(2_700);
  let target = Position::new(5, 2);
  let target_id = game
    .world_mut()
    .spawn_monster(target, "Static Target", 500, 100, (1, 7))
    .unwrap();
  let player_id = game.world().player_id().unwrap();

  let events = game
    .step(Command::AttackRangedChainfire(target))
    .expect("first BFG 10K chainfire burst should be accepted");

  assert_eq!(ranged_events(&events, player_id), 4);
  assert_eq!(
    events
      .iter()
      .filter(|event| matches!(
        event,
        GameEvent::AttackResolved {
          target_id: event_target,
          outcome: AttackOutcome::Hit { .. },
          is_ranged: true,
          ..
        } if *event_target == target_id
      ))
      .count(),
    4,
    "BFG 10K exact-hit chainfire should resolve four hits"
  );
  assert_eq!(bfg10k_schedules(&events, target_id), 4);

  let weapon = game.world().player().unwrap().equipment().weapon().unwrap();
  let props = weapon.weapon_properties().unwrap();
  assert_eq!(props.current_clip, 30);
  assert_eq!(props.chainfire_level, 1);
}

#[test]
fn bfg10k_second_chainfire_emits_five_exact_hits_and_advances_state() {
  let mut game = equipped_bfg10k(2_708);
  let initial_target = Position::new(5, 2);
  let target_id = game
    .world_mut()
    .spawn_monster(initial_target, "Static Target", 10_000, 100, (1, 7))
    .unwrap();

  game
    .step(Command::AttackRangedChainfire(initial_target))
    .expect("first BFG 10K chainfire burst should be accepted");
  let second_target = game
    .world()
    .get_actor(target_id)
    .expect("target should survive the first burst")
    .position();
  let player_id = game.world().player_id().unwrap();
  let events = game
    .step(Command::AttackRangedChainfire(second_target))
    .expect("second BFG 10K chainfire burst should be accepted");

  assert_eq!(ranged_events(&events, player_id), 5);
  assert_eq!(bfg10k_schedules(&events, target_id), 5);
  let weapon = game.world().player().unwrap().equipment().weapon().unwrap();
  let props = weapon.weapon_properties().unwrap();
  assert_eq!(props.current_clip, 5);
  assert_eq!(props.chainfire_level, 2);
}

#[test]
fn bfg10k_third_chainfire_emits_seven_exact_hits_and_advances_state() {
  let mut game = equipped_bfg10k(2_709);
  let target = Position::new(5, 2);
  let target_id = game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 7))
    .unwrap();

  game
    .step(Command::AttackRangedChainfire(target))
    .expect("first BFG 10K chainfire burst should be accepted");
  let second_target = game
    .world()
    .get_actor(target_id)
    .expect("target should survive the first burst")
    .position();
  game
    .step(Command::AttackRangedChainfire(second_target))
    .expect("second BFG 10K chainfire burst should be accepted");
  let third_target = game
    .world()
    .get_actor(target_id)
    .expect("target should survive the second burst")
    .position();
  let player_id = game.world().player_id().unwrap();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 35;

  let events = game
    .step(Command::AttackRangedChainfire(third_target))
    .expect("third BFG 10K chainfire burst should be accepted");

  assert_eq!(ranged_events(&events, player_id), 7);
  assert_eq!(bfg10k_schedules(&events, target_id), 7);
  let weapon = game.world().player().unwrap().equipment().weapon().unwrap();
  let props = weapon.weapon_properties().unwrap();
  assert_eq!(props.current_clip, 0);
  assert_eq!(props.chainfire_level, 3);
}

#[test]
fn bfg10k_fourth_chainfire_emits_seven_exact_hits_and_advances_state() {
  let mut game = equipped_bfg10k(2_711);
  let target = Position::new(8, 2);
  let target_id = game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 7))
    .unwrap();
  let player_id = game.world().player_id().unwrap();

  game
    .step(Command::AttackRangedChainfire(target))
    .expect("first BFG 10K chainfire burst should be accepted");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("second BFG 10K chainfire burst should be accepted");
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 35;
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("third BFG 10K chainfire burst should be accepted");
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 50;
  let events = game
    .step(Command::AttackRangedChainfire(target))
    .expect("fourth BFG 10K chainfire burst should be accepted");

  let weapon = game.world().player().unwrap().equipment().weapon().unwrap();
  let props = weapon.weapon_properties().unwrap();
  assert_eq!(props.current_clip, 15);
  assert_eq!(props.chainfire_level, 4);
  assert_eq!(ranged_events(&events, player_id), 7);
  assert_eq!(bfg10k_schedules(&events, target_id), 7);
}

#[test]
fn bfg10k_fifth_chainfire_emits_seven_exact_hits_and_advances_state() {
  let mut game = equipped_bfg10k(2_714);
  let target = Position::new(8, 2);
  let target_id = game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 7))
    .unwrap();
  let player_id = game.world().player_id().unwrap();

  game
    .step(Command::AttackRangedChainfire(target))
    .expect("first BFG 10K chainfire burst should be accepted");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("second BFG 10K chainfire burst should be accepted");
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 35;
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("third BFG 10K chainfire burst should be accepted");
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 50;
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("fourth BFG 10K chainfire burst should be accepted");
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 50;
  let events = game
    .step(Command::AttackRangedChainfire(target))
    .expect("fifth BFG 10K chainfire burst should be accepted");

  let weapon = game.world().player().unwrap().equipment().weapon().unwrap();
  let props = weapon.weapon_properties().unwrap();
  assert_eq!(props.current_clip, 15);
  assert_eq!(props.chainfire_level, 5);
  assert_eq!(ranged_events(&events, player_id), 7);
  assert_eq!(bfg10k_schedules(&events, target_id), 7);
}

#[test]
fn bfg10k_sixth_chainfire_emits_seven_exact_hits_and_advances_state() {
  let mut game = equipped_bfg10k(2_717);
  let target = Position::new(8, 2);
  let target_id = game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 7))
    .unwrap();
  let player_id = game.world().player_id().unwrap();

  game
    .step(Command::AttackRangedChainfire(target))
    .expect("first BFG 10K chainfire burst should be accepted");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("second BFG 10K chainfire burst should be accepted");
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 35;
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("third BFG 10K chainfire burst should be accepted");
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 50;
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("fourth BFG 10K chainfire burst should be accepted");
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 50;
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("fifth BFG 10K chainfire burst should be accepted");
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 50;
  let events = game
    .step(Command::AttackRangedChainfire(target))
    .expect("sixth BFG 10K chainfire burst should be accepted");

  let weapon = game.world().player().unwrap().equipment().weapon().unwrap();
  let props = weapon.weapon_properties().unwrap();
  assert_eq!(props.current_clip, 15);
  assert_eq!(props.chainfire_level, 6);
  assert_eq!(ranged_events(&events, player_id), 7);
  assert_eq!(bfg10k_schedules(&events, target_id), 7);
}

#[test]
fn bfg10k_seventh_chainfire_emits_seven_exact_hits_and_advances_state() {
  let mut game = equipped_bfg10k(2_720);
  let target = Position::new(8, 2);
  let target_id = game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 7))
    .unwrap();
  let player_id = game.world().player_id().unwrap();

  game
    .step(Command::AttackRangedChainfire(target))
    .expect("first BFG 10K chainfire burst should be accepted");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("second BFG 10K chainfire burst should be accepted");
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 35;
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("third BFG 10K chainfire burst should be accepted");
  for stage in ["fourth", "fifth", "sixth"] {
    game
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .weapon_mut()
      .unwrap()
      .weapon_properties_mut()
      .unwrap()
      .current_clip = 50;
    game
      .step(Command::AttackRangedChainfire(target))
      .unwrap_or_else(|error| {
        panic!("{stage} BFG 10K chainfire burst should be accepted: {error}")
      });
  }
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 50;
  let events = game
    .step(Command::AttackRangedChainfire(target))
    .expect("seventh BFG 10K chainfire burst should be accepted");

  let weapon = game.world().player().unwrap().equipment().weapon().unwrap();
  let props = weapon.weapon_properties().unwrap();
  assert_eq!(props.current_clip, 15);
  assert_eq!(props.chainfire_level, 7);
  assert_eq!(ranged_events(&events, player_id), 7);
  assert_eq!(bfg10k_schedules(&events, target_id), 7);
}

#[test]
fn bfg10k_eighth_chainfire_emits_seven_exact_hits_and_advances_state() {
  let mut game = equipped_bfg10k(2_722);
  let target = Position::new(8, 2);
  let target_id = game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 7))
    .unwrap();
  let player_id = game.world().player_id().unwrap();

  game
    .step(Command::AttackRangedChainfire(target))
    .expect("first BFG 10K chainfire burst should be accepted");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("second BFG 10K chainfire burst should be accepted");
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 35;
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("third BFG 10K chainfire burst should be accepted");
  for stage in ["fourth", "fifth", "sixth", "seventh"] {
    game
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .weapon_mut()
      .unwrap()
      .weapon_properties_mut()
      .unwrap()
      .current_clip = 50;
    game
      .step(Command::AttackRangedChainfire(target))
      .unwrap_or_else(|error| {
        panic!("{stage} BFG 10K chainfire burst should be accepted: {error}")
      });
  }
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 50;
  let events = game
    .step(Command::AttackRangedChainfire(target))
    .expect("eighth BFG 10K chainfire burst should be accepted");

  let weapon = game.world().player().unwrap().equipment().weapon().unwrap();
  let props = weapon.weapon_properties().unwrap();
  assert_eq!(props.current_clip, 15);
  assert_eq!(props.chainfire_level, 8);
  assert_eq!(ranged_events(&events, player_id), 7);
  assert_eq!(bfg10k_schedules(&events, target_id), 7);
}

#[test]
fn bfg10k_ninth_chainfire_emits_seven_exact_hits_and_advances_state() {
  let mut game = equipped_bfg10k(2_724);
  let target = Position::new(8, 2);
  let target_id = game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 7))
    .unwrap();
  let player_id = game.world().player_id().unwrap();

  game
    .step(Command::AttackRangedChainfire(target))
    .expect("first BFG 10K chainfire burst should be accepted");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("second BFG 10K chainfire burst should be accepted");
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 35;
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("third BFG 10K chainfire burst should be accepted");
  for stage in ["fourth", "fifth", "sixth", "seventh", "eighth"] {
    game
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .weapon_mut()
      .unwrap()
      .weapon_properties_mut()
      .unwrap()
      .current_clip = 50;
    game
      .step(Command::AttackRangedChainfire(target))
      .unwrap_or_else(|error| {
        panic!("{stage} BFG 10K chainfire burst should be accepted: {error}")
      });
  }
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 50;
  let events = game
    .step(Command::AttackRangedChainfire(target))
    .expect("ninth BFG 10K chainfire burst should be accepted");

  let weapon = game.world().player().unwrap().equipment().weapon().unwrap();
  let props = weapon.weapon_properties().unwrap();
  assert_eq!(props.current_clip, 15);
  assert_eq!(props.chainfire_level, 9);
  assert_eq!(ranged_events(&events, player_id), 7);
  assert_eq!(bfg10k_schedules(&events, target_id), 7);
}

#[test]
fn bfg10k_tenth_chainfire_emits_seven_exact_hits_and_advances_state() {
  let mut game = equipped_bfg10k(2_726);
  let target = Position::new(8, 2);
  let target_id = game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 7))
    .unwrap();
  let player_id = game.world().player_id().unwrap();

  game
    .step(Command::AttackRangedChainfire(target))
    .expect("first BFG 10K chainfire burst should be accepted");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("second BFG 10K chainfire burst should be accepted");
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 35;
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("third BFG 10K chainfire burst should be accepted");
  for stage in ["fourth", "fifth", "sixth", "seventh", "eighth", "ninth"] {
    game
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .weapon_mut()
      .unwrap()
      .weapon_properties_mut()
      .unwrap()
      .current_clip = 50;
    game
      .step(Command::AttackRangedChainfire(target))
      .unwrap_or_else(|error| {
        panic!("{stage} BFG 10K chainfire burst should be accepted: {error}")
      });
  }
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 50;
  let events = game
    .step(Command::AttackRangedChainfire(target))
    .expect("tenth BFG 10K chainfire burst should be accepted");

  let weapon = game.world().player().unwrap().equipment().weapon().unwrap();
  let props = weapon.weapon_properties().unwrap();
  assert_eq!(props.current_clip, 15);
  assert_eq!(props.chainfire_level, 10);
  assert_eq!(ranged_events(&events, player_id), 7);
  assert_eq!(bfg10k_schedules(&events, target_id), 7);
}

#[test]
fn bfg10k_eleventh_chainfire_emits_seven_exact_hits_and_advances_state() {
  let mut game = equipped_bfg10k(2_728);
  let target = Position::new(8, 2);
  let target_id = game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 7))
    .unwrap();
  let player_id = game.world().player_id().unwrap();

  game
    .step(Command::AttackRangedChainfire(target))
    .expect("first BFG 10K chainfire burst should be accepted");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("second BFG 10K chainfire burst should be accepted");
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 35;
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("third BFG 10K chainfire burst should be accepted");
  for stage in [
    "fourth", "fifth", "sixth", "seventh", "eighth", "ninth", "tenth",
  ] {
    game
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .weapon_mut()
      .unwrap()
      .weapon_properties_mut()
      .unwrap()
      .current_clip = 50;
    game
      .step(Command::AttackRangedChainfire(target))
      .unwrap_or_else(|error| {
        panic!("{stage} BFG 10K chainfire burst should be accepted: {error}")
      });
  }
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 50;
  let events = game
    .step(Command::AttackRangedChainfire(target))
    .expect("eleventh BFG 10K chainfire burst should be accepted");

  let weapon = game.world().player().unwrap().equipment().weapon().unwrap();
  let props = weapon.weapon_properties().unwrap();
  assert_eq!(props.current_clip, 15);
  assert_eq!(props.chainfire_level, 11);
  assert_eq!(ranged_events(&events, player_id), 7);
  assert_eq!(bfg10k_schedules(&events, target_id), 7);
}

#[test]
fn bfg10k_twelfth_chainfire_emits_seven_exact_hits_and_advances_state() {
  let mut game = equipped_bfg10k(2_730);
  let target = Position::new(8, 2);
  let target_id = game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 7))
    .unwrap();
  let player_id = game.world().player_id().unwrap();

  game
    .step(Command::AttackRangedChainfire(target))
    .expect("first BFG 10K chainfire burst should be accepted");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("second BFG 10K chainfire burst should be accepted");
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 35;
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("third BFG 10K chainfire burst should be accepted");
  for stage in [
    "fourth", "fifth", "sixth", "seventh", "eighth", "ninth", "tenth", "eleventh",
  ] {
    game
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .weapon_mut()
      .unwrap()
      .weapon_properties_mut()
      .unwrap()
      .current_clip = 50;
    game
      .step(Command::AttackRangedChainfire(target))
      .unwrap_or_else(|error| {
        panic!("{stage} BFG 10K chainfire burst should be accepted: {error}")
      });
  }
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 50;
  let events = game
    .step(Command::AttackRangedChainfire(target))
    .expect("twelfth BFG 10K chainfire burst should be accepted");

  let weapon = game.world().player().unwrap().equipment().weapon().unwrap();
  let props = weapon.weapon_properties().unwrap();
  assert_eq!(props.current_clip, 15);
  assert_eq!(props.chainfire_level, 12);
  assert_eq!(ranged_events(&events, player_id), 7);
  assert_eq!(bfg10k_schedules(&events, target_id), 7);
}

#[test]
fn bfg10k_thirteenth_chainfire_emits_seven_exact_hits_and_advances_state() {
  let mut game = equipped_bfg10k(2_732);
  let target = Position::new(8, 2);
  let target_id = game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 7))
    .unwrap();
  let player_id = game.world().player_id().unwrap();

  game
    .step(Command::AttackRangedChainfire(target))
    .expect("first BFG 10K chainfire burst should be accepted");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("second BFG 10K chainfire burst should be accepted");
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 35;
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("third BFG 10K chainfire burst should be accepted");
  for stage in [
    "fourth", "fifth", "sixth", "seventh", "eighth", "ninth", "tenth", "eleventh", "twelfth",
  ] {
    game
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .weapon_mut()
      .unwrap()
      .weapon_properties_mut()
      .unwrap()
      .current_clip = 50;
    game
      .step(Command::AttackRangedChainfire(target))
      .unwrap_or_else(|error| {
        panic!("{stage} BFG 10K chainfire burst should be accepted: {error}")
      });
  }
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 50;
  let events = game
    .step(Command::AttackRangedChainfire(target))
    .expect("thirteenth BFG 10K chainfire burst should be accepted");

  let weapon = game.world().player().unwrap().equipment().weapon().unwrap();
  let props = weapon.weapon_properties().unwrap();
  assert_eq!(props.current_clip, 15);
  assert_eq!(props.chainfire_level, 13);
  assert_eq!(ranged_events(&events, player_id), 7);
  assert_eq!(bfg10k_schedules(&events, target_id), 7);
}

#[test]
fn bfg10k_fourteenth_chainfire_emits_seven_exact_hits_and_advances_state() {
  let mut game = equipped_bfg10k(2_734);
  let target = Position::new(8, 2);
  let target_id = game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 7))
    .unwrap();
  let player_id = game.world().player_id().unwrap();

  game
    .step(Command::AttackRangedChainfire(target))
    .expect("first BFG 10K chainfire burst should be accepted");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("second BFG 10K chainfire burst should be accepted");
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 35;
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("third BFG 10K chainfire burst should be accepted");
  for stage in [
    "fourth",
    "fifth",
    "sixth",
    "seventh",
    "eighth",
    "ninth",
    "tenth",
    "eleventh",
    "twelfth",
    "thirteenth",
  ] {
    game
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .weapon_mut()
      .unwrap()
      .weapon_properties_mut()
      .unwrap()
      .current_clip = 50;
    game
      .step(Command::AttackRangedChainfire(target))
      .unwrap_or_else(|error| {
        panic!("{stage} BFG 10K chainfire burst should be accepted: {error}")
      });
  }
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 50;
  let events = game
    .step(Command::AttackRangedChainfire(target))
    .expect("fourteenth BFG 10K chainfire burst should be accepted");

  let weapon = game.world().player().unwrap().equipment().weapon().unwrap();
  let props = weapon.weapon_properties().unwrap();
  assert_eq!(props.current_clip, 15);
  assert_eq!(props.chainfire_level, 14);
  assert_eq!(ranged_events(&events, player_id), 7);
  assert_eq!(bfg10k_schedules(&events, target_id), 7);
}

#[test]
fn bfg10k_fifteenth_chainfire_emits_seven_exact_hits_and_advances_state() {
  let mut game = equipped_bfg10k(2_736);
  let target = Position::new(8, 2);
  let target_id = game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 7))
    .unwrap();
  let player_id = game.world().player_id().unwrap();

  game
    .step(Command::AttackRangedChainfire(target))
    .expect("first BFG 10K chainfire burst should be accepted");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("second BFG 10K chainfire burst should be accepted");
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 35;
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("third BFG 10K chainfire burst should be accepted");
  for stage in [
    "fourth",
    "fifth",
    "sixth",
    "seventh",
    "eighth",
    "ninth",
    "tenth",
    "eleventh",
    "twelfth",
    "thirteenth",
    "fourteenth",
  ] {
    game
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .weapon_mut()
      .unwrap()
      .weapon_properties_mut()
      .unwrap()
      .current_clip = 50;
    game
      .step(Command::AttackRangedChainfire(target))
      .unwrap_or_else(|error| {
        panic!("{stage} BFG 10K chainfire burst should be accepted: {error}")
      });
  }
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 50;
  let events = game
    .step(Command::AttackRangedChainfire(target))
    .expect("fifteenth BFG 10K chainfire burst should be accepted");

  let weapon = game.world().player().unwrap().equipment().weapon().unwrap();
  let props = weapon.weapon_properties().unwrap();
  assert_eq!(props.current_clip, 15);
  assert_eq!(props.chainfire_level, 15);
  assert_eq!(ranged_events(&events, player_id), 7);
  assert_eq!(bfg10k_schedules(&events, target_id), 7);
}

#[test]
fn bfg10k_chainfire_keeps_four_outcomes_after_lethal_target() {
  let mut game = equipped_bfg10k(2_701);
  let target = Position::new(5, 2);
  let target_id = game
    .world_mut()
    .spawn_monster(target, "Fragile Target", 1, 100, (1, 7))
    .unwrap();
  let player_id = game.world().player_id().unwrap();

  let events = game
    .step(Command::AttackRangedChainfire(target))
    .expect("BFG 10K chainfire against a visible target should be accepted");

  assert_eq!(ranged_events(&events, player_id), 4);
  assert_eq!(bfg10k_schedules(&events, target_id), 1);
  let death_index = events
    .iter()
    .position(
      |event| matches!(event, GameEvent::ActorDied { entity_id, .. } if *entity_id == target_id),
    )
    .expect("the exact-hit first projectile should kill the fragile target");
  assert!(!events[death_index + 1..].iter().any(|event| {
    matches!(
      event,
      GameEvent::DamageApplied {
        target_id: event_target,
        ..
      } if *event_target == target_id
    )
  }));
}

#[test]
fn bfg10k_chainfire_below_twenty_cell_cost_rejection_is_atomic() {
  let mut game = equipped_bfg10k(2_702);
  let target = Position::new(5, 2);
  game
    .world_mut()
    .spawn_monster(target, "Static Target", 500, 100, (1, 7))
    .unwrap();
  let player_id = game.world().player_id().unwrap();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 19;
  let before = game.clone();

  assert_eq!(
    game
      .step(Command::AttackRangedChainfire(target))
      .unwrap_err(),
    CommandError::NoAmmoInClip
  );
  assert_eq!(game, before);
}

#[test]
fn bfg10k_ordinary_fire_resets_chainfire_warmup() {
  let mut game = equipped_bfg10k(2_703);
  let target = Position::new(5, 2);
  let target_id = game
    .world_mut()
    .spawn_monster(target, "Static Target", 500, 0, (1, 7))
    .unwrap();
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("first BFG 10K chainfire burst");
  let player_id = game.world().player_id().unwrap();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 50;
  game
    .step(Command::AttackRanged(
      game.world().get_actor(target_id).unwrap().position(),
    ))
    .expect("ordinary fire after BFG 10K chainfire");

  assert_eq!(
    game
      .world()
      .player()
      .unwrap()
      .equipment()
      .weapon()
      .unwrap()
      .weapon_properties()
      .unwrap()
      .chainfire_level,
    0
  );
}

#[test]
fn bfg10k_sixteenth_chainfire_level_is_rejected_without_mutation() {
  let mut game = equipped_bfg10k(2_704);
  let target = Position::new(5, 2);
  let target_id = game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 7))
    .unwrap();
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("first BFG 10K chainfire burst");
  let second_target = game
    .world()
    .get_actor(target_id)
    .expect("target should survive the first burst")
    .position();
  game
    .step(Command::AttackRangedChainfire(second_target))
    .expect("second BFG 10K chainfire burst");
  let third_target = game
    .world()
    .get_actor(target_id)
    .expect("target should survive the second burst")
    .position();
  let player_id = game.world().player_id().unwrap();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 35;
  game
    .step(Command::AttackRangedChainfire(third_target))
    .expect("third BFG 10K chainfire burst");
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 50;
  game
    .step(Command::AttackRangedChainfire(third_target))
    .expect("fourth BFG 10K chainfire burst");
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 50;
  game
    .step(Command::AttackRangedChainfire(third_target))
    .expect("fifth BFG 10K chainfire burst");
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 50;
  game
    .step(Command::AttackRangedChainfire(third_target))
    .expect("sixth BFG 10K chainfire burst");
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 50;
  game
    .step(Command::AttackRangedChainfire(third_target))
    .expect("seventh BFG 10K chainfire burst");
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 50;
  game
    .step(Command::AttackRangedChainfire(third_target))
    .expect("eighth BFG 10K chainfire burst");
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 50;
  game
    .step(Command::AttackRangedChainfire(third_target))
    .expect("ninth BFG 10K chainfire burst");
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 50;
  game
    .step(Command::AttackRangedChainfire(third_target))
    .expect("tenth BFG 10K chainfire burst");
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 50;
  game
    .step(Command::AttackRangedChainfire(third_target))
    .expect("eleventh BFG 10K chainfire burst");
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 50;
  game
    .step(Command::AttackRangedChainfire(third_target))
    .expect("twelfth BFG 10K chainfire burst");
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 50;
  game
    .step(Command::AttackRangedChainfire(third_target))
    .expect("thirteenth BFG 10K chainfire burst");
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 50;
  game
    .step(Command::AttackRangedChainfire(third_target))
    .expect("fourteenth BFG 10K chainfire burst");
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 50;
  game
    .step(Command::AttackRangedChainfire(third_target))
    .expect("fifteenth BFG 10K chainfire burst");
  let before = game.clone();

  assert_eq!(
    game
      .step(Command::AttackRangedChainfire(
        game.world().get_actor(target_id).unwrap().position(),
      ))
      .unwrap_err(),
    CommandError::InvalidCommand("higher BFG 10K chainfire levels are deferred".to_string())
  );
  assert_eq!(game, before);
}

#[test]
fn bfg10k_third_chainfire_below_thirty_five_cell_cost_rejection_is_atomic() {
  let mut game = equipped_bfg10k(2_710);
  let target = Position::new(5, 2);
  let target_id = game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 7))
    .unwrap();
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("first BFG 10K chainfire burst");
  let second_target = game
    .world()
    .get_actor(target_id)
    .expect("target should survive the first burst")
    .position();
  game
    .step(Command::AttackRangedChainfire(second_target))
    .expect("second BFG 10K chainfire burst");
  let third_target = game
    .world()
    .get_actor(target_id)
    .expect("target should survive the second burst")
    .position();
  let player_id = game.world().player_id().unwrap();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 34;
  let before = game.clone();

  assert_eq!(
    game
      .step(Command::AttackRangedChainfire(third_target))
      .unwrap_err(),
    CommandError::NoAmmoInClip
  );
  assert_eq!(game, before);
}

#[test]
fn bfg10k_fourth_chainfire_below_thirty_five_cell_cost_rejection_is_atomic() {
  let mut game = equipped_bfg10k(2_712);
  let target = Position::new(8, 2);
  let target_id = game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 7))
    .unwrap();
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("first BFG 10K chainfire burst");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("second BFG 10K chainfire burst");
  let player_id = game.world().player_id().unwrap();
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 35;
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("third BFG 10K chainfire burst");
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 34;
  let before = game.clone();

  assert_eq!(
    game
      .step(Command::AttackRangedChainfire(
        game.world().get_actor(target_id).unwrap().position(),
      ))
      .unwrap_err(),
    CommandError::NoAmmoInClip
  );
  assert_eq!(game, before);
}

#[test]
fn bfg10k_fifth_chainfire_below_thirty_five_cell_cost_rejection_is_atomic() {
  let mut game = equipped_bfg10k(2_715);
  let target = Position::new(8, 2);
  game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 7))
    .unwrap();
  let player_id = game.world().player_id().unwrap();
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("first BFG 10K chainfire burst");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("second BFG 10K chainfire burst");
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 35;
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("third BFG 10K chainfire burst");
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 50;
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("fourth BFG 10K chainfire burst");
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 34;
  let before = game.clone();

  assert_eq!(
    game
      .step(Command::AttackRangedChainfire(target))
      .unwrap_err(),
    CommandError::NoAmmoInClip
  );
  assert_eq!(game, before);
}

#[test]
fn bfg10k_sixth_chainfire_below_thirty_five_cell_cost_rejection_is_atomic() {
  let mut game = equipped_bfg10k(2_718);
  let target = Position::new(8, 2);
  game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 7))
    .unwrap();
  let player_id = game.world().player_id().unwrap();
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("first BFG 10K chainfire burst");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("second BFG 10K chainfire burst");
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 35;
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("third BFG 10K chainfire burst");
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 50;
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("fourth BFG 10K chainfire burst");
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 50;
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("fifth BFG 10K chainfire burst");
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 34;
  let before = game.clone();

  assert_eq!(
    game
      .step(Command::AttackRangedChainfire(target))
      .unwrap_err(),
    CommandError::NoAmmoInClip
  );
  assert_eq!(game, before);
}

#[test]
fn bfg10k_seventh_chainfire_below_thirty_five_cell_cost_rejection_is_atomic() {
  let mut game = equipped_bfg10k(2_721);
  let target = Position::new(8, 2);
  game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 7))
    .unwrap();
  let player_id = game.world().player_id().unwrap();
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("first BFG 10K chainfire burst");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("second BFG 10K chainfire burst");
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 35;
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("third BFG 10K chainfire burst");
  for stage in ["fourth", "fifth", "sixth"] {
    game
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .weapon_mut()
      .unwrap()
      .weapon_properties_mut()
      .unwrap()
      .current_clip = 50;
    game
      .step(Command::AttackRangedChainfire(target))
      .unwrap_or_else(|error| panic!("{stage} BFG 10K chainfire burst: {error}"));
  }
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 34;
  let before = game.clone();

  assert_eq!(
    game
      .step(Command::AttackRangedChainfire(target))
      .unwrap_err(),
    CommandError::NoAmmoInClip
  );
  assert_eq!(game, before);
}

#[test]
fn bfg10k_tenth_chainfire_below_thirty_five_cell_cost_rejection_is_atomic() {
  let mut game = equipped_bfg10k(2_727);
  let target = Position::new(8, 2);
  game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 7))
    .unwrap();
  let player_id = game.world().player_id().unwrap();
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("first BFG 10K chainfire burst");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("second BFG 10K chainfire burst");
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 35;
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("third BFG 10K chainfire burst");
  for stage in ["fourth", "fifth", "sixth", "seventh", "eighth", "ninth"] {
    game
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .weapon_mut()
      .unwrap()
      .weapon_properties_mut()
      .unwrap()
      .current_clip = 50;
    game
      .step(Command::AttackRangedChainfire(target))
      .unwrap_or_else(|error| panic!("{stage} BFG 10K chainfire burst: {error}"));
  }
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 34;
  let before = game.clone();

  assert_eq!(
    game
      .step(Command::AttackRangedChainfire(target))
      .unwrap_err(),
    CommandError::NoAmmoInClip
  );
  assert_eq!(game, before);
}

#[test]
fn bfg10k_eleventh_chainfire_below_thirty_five_cell_cost_rejection_is_atomic() {
  let mut game = equipped_bfg10k(2_729);
  let target = Position::new(8, 2);
  game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 7))
    .unwrap();
  let player_id = game.world().player_id().unwrap();
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("first BFG 10K chainfire burst");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("second BFG 10K chainfire burst");
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 35;
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("third BFG 10K chainfire burst");
  for stage in [
    "fourth", "fifth", "sixth", "seventh", "eighth", "ninth", "tenth",
  ] {
    game
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .weapon_mut()
      .unwrap()
      .weapon_properties_mut()
      .unwrap()
      .current_clip = 50;
    game
      .step(Command::AttackRangedChainfire(target))
      .unwrap_or_else(|error| panic!("{stage} BFG 10K chainfire burst: {error}"));
  }
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 34;
  let before = game.clone();

  assert_eq!(
    game
      .step(Command::AttackRangedChainfire(target))
      .unwrap_err(),
    CommandError::NoAmmoInClip
  );
  assert_eq!(game, before);
}

#[test]
fn bfg10k_twelfth_chainfire_below_thirty_five_cell_cost_rejection_is_atomic() {
  let mut game = equipped_bfg10k(2_731);
  let target = Position::new(8, 2);
  game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 7))
    .unwrap();
  let player_id = game.world().player_id().unwrap();
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("first BFG 10K chainfire burst");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("second BFG 10K chainfire burst");
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 35;
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("third BFG 10K chainfire burst");
  for stage in [
    "fourth", "fifth", "sixth", "seventh", "eighth", "ninth", "tenth", "eleventh",
  ] {
    game
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .weapon_mut()
      .unwrap()
      .weapon_properties_mut()
      .unwrap()
      .current_clip = 50;
    game
      .step(Command::AttackRangedChainfire(target))
      .unwrap_or_else(|error| panic!("{stage} BFG 10K chainfire burst: {error}"));
  }
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 34;
  let before = game.clone();

  assert_eq!(
    game
      .step(Command::AttackRangedChainfire(target))
      .unwrap_err(),
    CommandError::NoAmmoInClip
  );
  assert_eq!(game, before);
}

#[test]
fn bfg10k_thirteenth_chainfire_below_thirty_five_cell_cost_rejection_is_atomic() {
  let mut game = equipped_bfg10k(2_733);
  let target = Position::new(8, 2);
  game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 7))
    .unwrap();
  let player_id = game.world().player_id().unwrap();
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("first BFG 10K chainfire burst");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("second BFG 10K chainfire burst");
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 35;
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("third BFG 10K chainfire burst");
  for stage in [
    "fourth", "fifth", "sixth", "seventh", "eighth", "ninth", "tenth", "eleventh", "twelfth",
  ] {
    game
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .weapon_mut()
      .unwrap()
      .weapon_properties_mut()
      .unwrap()
      .current_clip = 50;
    game
      .step(Command::AttackRangedChainfire(target))
      .unwrap_or_else(|error| panic!("{stage} BFG 10K chainfire burst: {error}"));
  }
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 34;
  let before = game.clone();

  assert_eq!(
    game
      .step(Command::AttackRangedChainfire(target))
      .unwrap_err(),
    CommandError::NoAmmoInClip
  );
  assert_eq!(game, before);
}

#[test]
fn bfg10k_fourteenth_chainfire_below_thirty_five_cell_cost_rejection_is_atomic() {
  let mut game = equipped_bfg10k(2_735);
  let target = Position::new(8, 2);
  game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 7))
    .unwrap();
  let player_id = game.world().player_id().unwrap();
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("first BFG 10K chainfire burst");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("second BFG 10K chainfire burst");
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 35;
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("third BFG 10K chainfire burst");
  for stage in [
    "fourth",
    "fifth",
    "sixth",
    "seventh",
    "eighth",
    "ninth",
    "tenth",
    "eleventh",
    "twelfth",
    "thirteenth",
  ] {
    game
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .weapon_mut()
      .unwrap()
      .weapon_properties_mut()
      .unwrap()
      .current_clip = 50;
    game
      .step(Command::AttackRangedChainfire(target))
      .unwrap_or_else(|error| panic!("{stage} BFG 10K chainfire burst: {error}"));
  }
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 34;
  let before = game.clone();

  assert_eq!(
    game
      .step(Command::AttackRangedChainfire(target))
      .unwrap_err(),
    CommandError::NoAmmoInClip
  );
  assert_eq!(game, before);
}

#[test]
fn bfg10k_fifteenth_chainfire_below_thirty_five_cell_cost_rejection_is_atomic() {
  let mut game = equipped_bfg10k(2_737);
  let target = Position::new(8, 2);
  game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 7))
    .unwrap();
  let player_id = game.world().player_id().unwrap();
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("first BFG 10K chainfire burst");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("second BFG 10K chainfire burst");
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 35;
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("third BFG 10K chainfire burst");
  for stage in [
    "fourth",
    "fifth",
    "sixth",
    "seventh",
    "eighth",
    "ninth",
    "tenth",
    "eleventh",
    "twelfth",
    "thirteenth",
    "fourteenth",
  ] {
    game
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .weapon_mut()
      .unwrap()
      .weapon_properties_mut()
      .unwrap()
      .current_clip = 50;
    game
      .step(Command::AttackRangedChainfire(target))
      .unwrap_or_else(|error| panic!("{stage} BFG 10K chainfire burst: {error}"));
  }
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 34;
  let before = game.clone();

  assert_eq!(
    game
      .step(Command::AttackRangedChainfire(target))
      .unwrap_err(),
    CommandError::NoAmmoInClip
  );
  assert_eq!(game, before);
}

#[test]
fn bfg10k_eighth_chainfire_below_thirty_five_cell_cost_rejection_is_atomic() {
  let mut game = equipped_bfg10k(2_723);
  let target = Position::new(8, 2);
  game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 7))
    .unwrap();
  let player_id = game.world().player_id().unwrap();
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("first BFG 10K chainfire burst");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("second BFG 10K chainfire burst");
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 35;
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("third BFG 10K chainfire burst");
  for stage in ["fourth", "fifth", "sixth", "seventh"] {
    game
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .weapon_mut()
      .unwrap()
      .weapon_properties_mut()
      .unwrap()
      .current_clip = 50;
    game
      .step(Command::AttackRangedChainfire(target))
      .unwrap_or_else(|error| panic!("{stage} BFG 10K chainfire burst: {error}"));
  }
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 34;
  let before = game.clone();

  assert_eq!(
    game
      .step(Command::AttackRangedChainfire(target))
      .unwrap_err(),
    CommandError::NoAmmoInClip
  );
  assert_eq!(game, before);
}

#[test]
fn bfg10k_ninth_chainfire_below_thirty_five_cell_cost_rejection_is_atomic() {
  let mut game = equipped_bfg10k(2_725);
  let target = Position::new(8, 2);
  game
    .world_mut()
    .spawn_monster(target, "Static Target", 10_000, 0, (1, 7))
    .unwrap();
  let player_id = game.world().player_id().unwrap();
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("first BFG 10K chainfire burst");
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("second BFG 10K chainfire burst");
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 35;
  game
    .step(Command::AttackRangedChainfire(target))
    .expect("third BFG 10K chainfire burst");
  for stage in ["fourth", "fifth", "sixth", "seventh", "eighth"] {
    game
      .world_mut()
      .get_actor_mut(player_id)
      .unwrap()
      .equipment_mut()
      .weapon_mut()
      .unwrap()
      .weapon_properties_mut()
      .unwrap()
      .current_clip = 50;
    game
      .step(Command::AttackRangedChainfire(target))
      .unwrap_or_else(|error| panic!("{stage} BFG 10K chainfire burst: {error}"));
  }
  game
    .world_mut()
    .get_actor_mut(player_id)
    .unwrap()
    .equipment_mut()
    .weapon_mut()
    .unwrap()
    .weapon_properties_mut()
    .unwrap()
    .current_clip = 34;
  let before = game.clone();

  assert_eq!(
    game
      .step(Command::AttackRangedChainfire(target))
      .unwrap_err(),
    CommandError::NoAmmoInClip
  );
  assert_eq!(game, before);
}

#[test]
fn bfg10k_chainfire_replay_is_deterministic() {
  let player_start = Position::new(5, 5);
  let mut replay =
    ReplayLog::new(2_705, 12, 12, player_start).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: vec![ItemSpawnKind::AmmoCells(45)],
      equipped_weapon: Some(ItemSpawnKind::Bfg10k),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  let target = Position::new(11, 5);
  replay.record_monster(MonsterSpawnSpec::new(target, "Target", 10_000, 0, (1, 7)));
  replay.record_command(Command::AttackRangedChainfire(target));
  replay.record_command(Command::AttackRangedChainfire(target));
  replay.record_command(Command::Reload);
  replay.record_command(Command::AttackRangedChainfire(target));

  let (game, events) = ReplayEngine::run(&replay).expect("BFG 10K chainfire replay should run");
  let weapon = game.world().player().unwrap().equipment().weapon().unwrap();
  assert_eq!(weapon.weapon_properties().unwrap().current_clip, 15);
  assert_eq!(weapon.weapon_properties().unwrap().chainfire_level, 3);
  assert_eq!(
    events
      .iter()
      .filter(|event| matches!(
        event,
        GameEvent::AttackResolved {
          is_ranged: true,
          ..
        }
      ))
      .count(),
    16
  );
  assert_eq!(
    events
      .iter()
      .filter(|event| matches!(event, GameEvent::Bfg10kExplosionScheduled { .. }))
      .count(),
    16
  );
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn bfg10k_fourth_chainfire_replay_is_deterministic() {
  let player_start = Position::new(5, 5);
  let mut replay =
    ReplayLog::new(2_713, 12, 12, player_start).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: vec![ItemSpawnKind::AmmoCells(150)],
      equipped_weapon: Some(ItemSpawnKind::Bfg10k),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  let target = Position::new(11, 5);
  replay.record_monster(MonsterSpawnSpec::new(target, "Target", 10_000, 0, (1, 7)));
  replay.record_command(Command::AttackRangedChainfire(target));
  replay.record_command(Command::AttackRangedChainfire(target));
  replay.record_command(Command::Reload);
  replay.record_command(Command::AttackRangedChainfire(target));
  replay.record_command(Command::Reload);
  replay.record_command(Command::AttackRangedChainfire(target));

  let (game, events) =
    ReplayEngine::run(&replay).expect("BFG 10K fourth chainfire replay should run");
  let weapon = game.world().player().unwrap().equipment().weapon().unwrap();
  let props = weapon.weapon_properties().unwrap();
  assert_eq!(props.current_clip, 15);
  assert_eq!(props.chainfire_level, 4);
  assert_eq!(
    events
      .iter()
      .filter(|event| matches!(
        event,
        GameEvent::AttackResolved {
          is_ranged: true,
          ..
        }
      ))
      .count(),
    23
  );
  assert_eq!(
    events
      .iter()
      .filter(|event| matches!(event, GameEvent::Bfg10kExplosionScheduled { .. }))
      .count(),
    23
  );
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn bfg10k_fifth_chainfire_replay_is_deterministic() {
  let player_start = Position::new(5, 5);
  let mut replay =
    ReplayLog::new(2_716, 12, 12, player_start).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: vec![ItemSpawnKind::AmmoCells(150)],
      equipped_weapon: Some(ItemSpawnKind::Bfg10k),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  let target = Position::new(11, 5);
  replay.record_monster(MonsterSpawnSpec::new(target, "Target", 10_000, 0, (1, 7)));
  replay.record_command(Command::AttackRangedChainfire(target));
  replay.record_command(Command::AttackRangedChainfire(target));
  replay.record_command(Command::Reload);
  replay.record_command(Command::AttackRangedChainfire(target));
  replay.record_command(Command::Reload);
  replay.record_command(Command::AttackRangedChainfire(target));
  replay.record_command(Command::Reload);
  replay.record_command(Command::AttackRangedChainfire(target));

  let (game, events) =
    ReplayEngine::run(&replay).expect("BFG 10K fifth chainfire replay should run");
  let weapon = game.world().player().unwrap().equipment().weapon().unwrap();
  let props = weapon.weapon_properties().unwrap();
  assert_eq!(props.current_clip, 15);
  assert_eq!(props.chainfire_level, 5);
  assert_eq!(
    events
      .iter()
      .filter(|event| matches!(
        event,
        GameEvent::AttackResolved {
          is_ranged: true,
          ..
        }
      ))
      .count(),
    30
  );
  assert_eq!(
    events
      .iter()
      .filter(|event| matches!(event, GameEvent::Bfg10kExplosionScheduled { .. }))
      .count(),
    30
  );
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn bfg10k_sixth_chainfire_replay_is_deterministic() {
  let player_start = Position::new(5, 5);
  let mut replay =
    ReplayLog::new(2_719, 12, 12, player_start).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: vec![ItemSpawnKind::AmmoCells(150)],
      equipped_weapon: Some(ItemSpawnKind::Bfg10k),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  let target = Position::new(11, 5);
  replay.record_monster(MonsterSpawnSpec::new(target, "Target", 10_000, 0, (1, 7)));
  replay.record_command(Command::AttackRangedChainfire(target));
  replay.record_command(Command::AttackRangedChainfire(target));
  replay.record_command(Command::Reload);
  replay.record_command(Command::AttackRangedChainfire(target));
  replay.record_command(Command::Reload);
  replay.record_command(Command::AttackRangedChainfire(target));
  replay.record_command(Command::Reload);
  replay.record_command(Command::AttackRangedChainfire(target));
  replay.record_command(Command::Reload);
  replay.record_command(Command::AttackRangedChainfire(target));

  let (game, events) =
    ReplayEngine::run(&replay).expect("BFG 10K sixth chainfire replay should run");
  let weapon = game.world().player().unwrap().equipment().weapon().unwrap();
  let props = weapon.weapon_properties().unwrap();
  assert_eq!(props.current_clip, 15);
  assert_eq!(props.chainfire_level, 6);
  assert_eq!(
    events
      .iter()
      .filter(|event| matches!(
        event,
        GameEvent::AttackResolved {
          is_ranged: true,
          ..
        }
      ))
      .count(),
    37
  );
  assert_eq!(
    events
      .iter()
      .filter(|event| matches!(event, GameEvent::Bfg10kExplosionScheduled { .. }))
      .count(),
    37
  );
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn bfg10k_seventh_chainfire_replay_is_deterministic() {
  let player_start = Position::new(5, 5);
  let mut replay =
    ReplayLog::new(2_720, 12, 12, player_start).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: vec![ItemSpawnKind::AmmoCells(220)],
      equipped_weapon: Some(ItemSpawnKind::Bfg10k),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  let target = Position::new(11, 5);
  replay.record_monster(MonsterSpawnSpec::new(target, "Target", 10_000, 0, (1, 7)));
  replay.record_command(Command::AttackRangedChainfire(target));
  replay.record_command(Command::AttackRangedChainfire(target));
  replay.record_command(Command::Reload);
  replay.record_command(Command::AttackRangedChainfire(target));
  replay.record_command(Command::Reload);
  replay.record_command(Command::AttackRangedChainfire(target));
  replay.record_command(Command::Reload);
  replay.record_command(Command::AttackRangedChainfire(target));
  replay.record_command(Command::Reload);
  replay.record_command(Command::AttackRangedChainfire(target));
  replay.record_command(Command::Reload);
  replay.record_command(Command::AttackRangedChainfire(target));

  let (game, events) =
    ReplayEngine::run(&replay).expect("BFG 10K seventh chainfire replay should run");
  let weapon = game.world().player().unwrap().equipment().weapon().unwrap();
  let props = weapon.weapon_properties().unwrap();
  assert_eq!(props.current_clip, 15);
  assert_eq!(props.chainfire_level, 7);
  assert_eq!(
    events
      .iter()
      .filter(|event| matches!(
        event,
        GameEvent::AttackResolved {
          is_ranged: true,
          ..
        }
      ))
      .count(),
    44
  );
  assert_eq!(
    events
      .iter()
      .filter(|event| matches!(event, GameEvent::Bfg10kExplosionScheduled { .. }))
      .count(),
    44
  );
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn bfg10k_eighth_chainfire_replay_is_deterministic() {
  let player_start = Position::new(5, 5);
  let mut replay =
    ReplayLog::new(2_722, 12, 12, player_start).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: vec![ItemSpawnKind::AmmoCells(255)],
      equipped_weapon: Some(ItemSpawnKind::Bfg10k),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  let target = Position::new(11, 5);
  replay.record_monster(MonsterSpawnSpec::new(target, "Target", 10_000, 0, (1, 7)));
  replay.record_command(Command::AttackRangedChainfire(target));
  replay.record_command(Command::AttackRangedChainfire(target));
  replay.record_command(Command::Reload);
  replay.record_command(Command::AttackRangedChainfire(target));
  replay.record_command(Command::Reload);
  replay.record_command(Command::AttackRangedChainfire(target));
  replay.record_command(Command::Reload);
  replay.record_command(Command::AttackRangedChainfire(target));
  replay.record_command(Command::Reload);
  replay.record_command(Command::AttackRangedChainfire(target));
  replay.record_command(Command::Reload);
  replay.record_command(Command::AttackRangedChainfire(target));
  replay.record_command(Command::Reload);
  replay.record_command(Command::AttackRangedChainfire(target));

  let (game, events) =
    ReplayEngine::run(&replay).expect("BFG 10K eighth chainfire replay should run");
  let weapon = game.world().player().unwrap().equipment().weapon().unwrap();
  let props = weapon.weapon_properties().unwrap();
  assert_eq!(props.current_clip, 15);
  assert_eq!(props.chainfire_level, 8);
  assert_eq!(
    events
      .iter()
      .filter(|event| matches!(
        event,
        GameEvent::AttackResolved {
          is_ranged: true,
          ..
        }
      ))
      .count(),
    51
  );
  assert_eq!(
    events
      .iter()
      .filter(|event| matches!(event, GameEvent::Bfg10kExplosionScheduled { .. }))
      .count(),
    51
  );
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn bfg10k_ninth_chainfire_replay_is_deterministic() {
  let player_start = Position::new(5, 5);
  let mut replay =
    ReplayLog::new(2_724, 12, 12, player_start).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: vec![ItemSpawnKind::AmmoCells(290)],
      equipped_weapon: Some(ItemSpawnKind::Bfg10k),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  let target = Position::new(11, 5);
  replay.record_monster(MonsterSpawnSpec::new(target, "Target", 10_000, 0, (1, 7)));
  replay.record_command(Command::AttackRangedChainfire(target));
  replay.record_command(Command::AttackRangedChainfire(target));
  replay.record_command(Command::Reload);
  replay.record_command(Command::AttackRangedChainfire(target));
  replay.record_command(Command::Reload);
  replay.record_command(Command::AttackRangedChainfire(target));
  replay.record_command(Command::Reload);
  replay.record_command(Command::AttackRangedChainfire(target));
  replay.record_command(Command::Reload);
  replay.record_command(Command::AttackRangedChainfire(target));
  replay.record_command(Command::Reload);
  replay.record_command(Command::AttackRangedChainfire(target));
  replay.record_command(Command::Reload);
  replay.record_command(Command::AttackRangedChainfire(target));
  replay.record_command(Command::Reload);
  replay.record_command(Command::AttackRangedChainfire(target));

  let (game, events) =
    ReplayEngine::run(&replay).expect("BFG 10K ninth chainfire replay should run");
  let weapon = game.world().player().unwrap().equipment().weapon().unwrap();
  let props = weapon.weapon_properties().unwrap();
  assert_eq!(props.current_clip, 15);
  assert_eq!(props.chainfire_level, 9);
  assert_eq!(
    events
      .iter()
      .filter(|event| matches!(
        event,
        GameEvent::AttackResolved {
          is_ranged: true,
          ..
        }
      ))
      .count(),
    58
  );
  assert_eq!(
    events
      .iter()
      .filter(|event| matches!(event, GameEvent::Bfg10kExplosionScheduled { .. }))
      .count(),
    58
  );
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn bfg10k_tenth_chainfire_replay_is_deterministic() {
  let player_start = Position::new(5, 5);
  let mut replay =
    ReplayLog::new(2_726, 12, 12, player_start).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: vec![ItemSpawnKind::AmmoCells(325)],
      equipped_weapon: Some(ItemSpawnKind::Bfg10k),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  let target = Position::new(11, 5);
  replay.record_monster(MonsterSpawnSpec::new(target, "Target", 10_000, 0, (1, 7)));
  replay.record_command(Command::AttackRangedChainfire(target));
  replay.record_command(Command::AttackRangedChainfire(target));
  for _ in 0..8 {
    replay.record_command(Command::Reload);
    replay.record_command(Command::AttackRangedChainfire(target));
  }

  let (game, events) =
    ReplayEngine::run(&replay).expect("BFG 10K tenth chainfire replay should run");
  let weapon = game.world().player().unwrap().equipment().weapon().unwrap();
  let props = weapon.weapon_properties().unwrap();
  assert_eq!(props.current_clip, 15);
  assert_eq!(props.chainfire_level, 10);
  assert_eq!(
    events
      .iter()
      .filter(|event| matches!(
        event,
        GameEvent::AttackResolved {
          is_ranged: true,
          ..
        }
      ))
      .count(),
    65
  );
  assert_eq!(
    events
      .iter()
      .filter(|event| matches!(event, GameEvent::Bfg10kExplosionScheduled { .. }))
      .count(),
    65
  );
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn bfg10k_eleventh_chainfire_replay_is_deterministic() {
  let player_start = Position::new(5, 5);
  let mut replay =
    ReplayLog::new(2_728, 12, 12, player_start).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: vec![ItemSpawnKind::AmmoCells(360)],
      equipped_weapon: Some(ItemSpawnKind::Bfg10k),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  let target = Position::new(11, 5);
  replay.record_monster(MonsterSpawnSpec::new(target, "Target", 10_000, 0, (1, 7)));
  replay.record_command(Command::AttackRangedChainfire(target));
  replay.record_command(Command::AttackRangedChainfire(target));
  for _ in 0..9 {
    replay.record_command(Command::Reload);
    replay.record_command(Command::AttackRangedChainfire(target));
  }

  let (game, events) =
    ReplayEngine::run(&replay).expect("BFG 10K eleventh chainfire replay should run");
  let weapon = game.world().player().unwrap().equipment().weapon().unwrap();
  let props = weapon.weapon_properties().unwrap();
  assert_eq!(props.current_clip, 15);
  assert_eq!(props.chainfire_level, 11);
  assert_eq!(
    events
      .iter()
      .filter(|event| matches!(
        event,
        GameEvent::AttackResolved {
          is_ranged: true,
          ..
        }
      ))
      .count(),
    72
  );
  assert_eq!(
    events
      .iter()
      .filter(|event| matches!(event, GameEvent::Bfg10kExplosionScheduled { .. }))
      .count(),
    72
  );
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn bfg10k_twelfth_chainfire_replay_is_deterministic() {
  let player_start = Position::new(5, 5);
  let mut replay =
    ReplayLog::new(2_730, 12, 12, player_start).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: vec![ItemSpawnKind::AmmoCells(395)],
      equipped_weapon: Some(ItemSpawnKind::Bfg10k),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  let target = Position::new(11, 5);
  replay.record_monster(MonsterSpawnSpec::new(target, "Target", 10_000, 0, (1, 7)));
  replay.record_command(Command::AttackRangedChainfire(target));
  replay.record_command(Command::AttackRangedChainfire(target));
  for _ in 0..10 {
    replay.record_command(Command::Reload);
    replay.record_command(Command::AttackRangedChainfire(target));
  }

  let (game, events) =
    ReplayEngine::run(&replay).expect("BFG 10K twelfth chainfire replay should run");
  let weapon = game.world().player().unwrap().equipment().weapon().unwrap();
  let props = weapon.weapon_properties().unwrap();
  assert_eq!(props.current_clip, 15);
  assert_eq!(props.chainfire_level, 12);
  assert_eq!(
    events
      .iter()
      .filter(|event| matches!(
        event,
        GameEvent::AttackResolved {
          is_ranged: true,
          ..
        }
      ))
      .count(),
    79
  );
  assert_eq!(
    events
      .iter()
      .filter(|event| matches!(event, GameEvent::Bfg10kExplosionScheduled { .. }))
      .count(),
    79
  );
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn bfg10k_thirteenth_chainfire_replay_is_deterministic() {
  let player_start = Position::new(5, 5);
  let mut replay =
    ReplayLog::new(2_732, 12, 12, player_start).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: vec![ItemSpawnKind::AmmoCells(430)],
      equipped_weapon: Some(ItemSpawnKind::Bfg10k),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  let target = Position::new(11, 5);
  replay.record_monster(MonsterSpawnSpec::new(target, "Target", 10_000, 0, (1, 7)));
  replay.record_command(Command::AttackRangedChainfire(target));
  replay.record_command(Command::AttackRangedChainfire(target));
  for _ in 0..11 {
    replay.record_command(Command::Reload);
    replay.record_command(Command::AttackRangedChainfire(target));
  }

  let (game, events) =
    ReplayEngine::run(&replay).expect("BFG 10K thirteenth chainfire replay should run");
  let weapon = game.world().player().unwrap().equipment().weapon().unwrap();
  let props = weapon.weapon_properties().unwrap();
  assert_eq!(props.current_clip, 15);
  assert_eq!(props.chainfire_level, 13);
  assert_eq!(
    events
      .iter()
      .filter(|event| matches!(
        event,
        GameEvent::AttackResolved {
          is_ranged: true,
          ..
        }
      ))
      .count(),
    86
  );
  assert_eq!(
    events
      .iter()
      .filter(|event| matches!(event, GameEvent::Bfg10kExplosionScheduled { .. }))
      .count(),
    86
  );
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn bfg10k_fourteenth_chainfire_replay_is_deterministic() {
  let player_start = Position::new(5, 5);
  let mut replay =
    ReplayLog::new(2_734, 12, 12, player_start).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: vec![ItemSpawnKind::AmmoCells(465)],
      equipped_weapon: Some(ItemSpawnKind::Bfg10k),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  let target = Position::new(11, 5);
  replay.record_monster(MonsterSpawnSpec::new(target, "Target", 10_000, 0, (1, 7)));
  replay.record_command(Command::AttackRangedChainfire(target));
  replay.record_command(Command::AttackRangedChainfire(target));
  for _ in 0..12 {
    replay.record_command(Command::Reload);
    replay.record_command(Command::AttackRangedChainfire(target));
  }

  let (game, events) =
    ReplayEngine::run(&replay).expect("BFG 10K fourteenth chainfire replay should run");
  let weapon = game.world().player().unwrap().equipment().weapon().unwrap();
  let props = weapon.weapon_properties().unwrap();
  assert_eq!(props.current_clip, 15);
  assert_eq!(props.chainfire_level, 14);
  assert_eq!(
    events
      .iter()
      .filter(|event| matches!(
        event,
        GameEvent::AttackResolved {
          is_ranged: true,
          ..
        }
      ))
      .count(),
    93
  );
  assert_eq!(
    events
      .iter()
      .filter(|event| matches!(event, GameEvent::Bfg10kExplosionScheduled { .. }))
      .count(),
    93
  );
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}

#[test]
fn bfg10k_fifteenth_chainfire_replay_is_deterministic() {
  let player_start = Position::new(5, 5);
  let mut replay =
    ReplayLog::new(2_736, 12, 12, player_start).with_player_config(PlayerSpawnConfig {
      hp: 50,
      max_hp: 50,
      speed: 100,
      initial_items: vec![ItemSpawnKind::AmmoCells(500)],
      equipped_weapon: Some(ItemSpawnKind::Bfg10k),
      equipped_armor: None,
      equipped_armor_durability: None,
    });
  let target = Position::new(11, 5);
  replay.record_monster(MonsterSpawnSpec::new(target, "Target", 10_000, 0, (1, 7)));
  replay.record_command(Command::AttackRangedChainfire(target));
  replay.record_command(Command::AttackRangedChainfire(target));
  for _ in 0..13 {
    replay.record_command(Command::Reload);
    replay.record_command(Command::AttackRangedChainfire(target));
  }

  let (game, events) =
    ReplayEngine::run(&replay).expect("BFG 10K fifteenth chainfire replay should run");
  let weapon = game.world().player().unwrap().equipment().weapon().unwrap();
  let props = weapon.weapon_properties().unwrap();
  assert_eq!(props.current_clip, 15);
  assert_eq!(props.chainfire_level, 15);
  assert_eq!(
    events
      .iter()
      .filter(|event| matches!(
        event,
        GameEvent::AttackResolved {
          is_ranged: true,
          ..
        }
      ))
      .count(),
    100
  );
  assert_eq!(
    events
      .iter()
      .filter(|event| matches!(event, GameEvent::Bfg10kExplosionScheduled { .. }))
      .count(),
    100
  );
  assert!(ReplayEngine::verify_determinism(&replay).unwrap());
}
