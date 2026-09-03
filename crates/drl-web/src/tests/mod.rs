//! Browser-boundary test modules grouped by responsibility.
//! Each module keeps the original `use super::*` boundary so the tests
//! still exercise the crate surface rather than a second harness.

mod animation;
mod assets;
mod chainfire_parity;
mod content;
mod core_parity;
mod heavy_weapon_parity;
mod input;
mod markup;
mod session;
mod snapshots;
mod weapon_parity;

use crate::*;
use drl_assets::{AtlasId, SpriteUv};
use drl_core::item::Item;
use drl_core::{Game, Tile};
use drl_protocol::{
  Command, Direction, ItemArchetype, ItemCategory, ItemId, ItemSpawnKind, ItemSpawnSpec, ItemView,
  MonsterSpawnSpec, PlayerSpawnConfig, Position, ReplayLog, TileKind,
};
use drl_render::{
  LightingBand, MinimapMarker, MinimapState, PixelRect, RenderScene,
  effect_timeline_for_observations,
};

type ChainfireWeaponCase = (ItemArchetype, fn(ItemId) -> Item, u32, u32);

fn assert_bfg10k_volley_events(
  events: &[drl_protocol::GameEvent],
  attacker_id: drl_protocol::EntityId,
  target_id: drl_protocol::EntityId,
) {
  let mut attacks = Vec::new();
  let mut damages = Vec::new();
  let mut schedules = Vec::new();
  for (index, event) in events.iter().enumerate() {
    match event {
      drl_protocol::GameEvent::AttackResolved {
        attacker_id: event_attacker,
        target_id: event_target,
        outcome: drl_protocol::AttackOutcome::Hit { damage, .. },
        is_ranged: true,
      } if *event_attacker == attacker_id && *event_target == target_id => {
        attacks.push((index, *damage));
      }
      drl_protocol::GameEvent::DamageApplied {
        target_id: event_target,
        amount,
        source: drl_protocol::DamageSource::Actor(_),
        damage_type: Some(drl_protocol::DamageType::Plasma),
        ..
      } if *event_target == target_id => damages.push((index, *amount)),
      drl_protocol::GameEvent::Bfg10kExplosionScheduled {
        entity_id,
        target_id: event_target,
        delay,
        radius,
        knockback,
      } if *entity_id == attacker_id && *event_target == target_id => {
        schedules.push((index, *delay, *radius, *knockback));
      }
      _ => {}
    }
  }

  assert_eq!(attacks.len(), 5, "BFG 10K volley must resolve five hits");
  assert_eq!(
    damages.len(),
    5,
    "BFG 10K volley must apply five damage events"
  );
  assert_eq!(
    attacks
      .iter()
      .map(|(_, damage)| *damage)
      .collect::<Vec<_>>(),
    damages
      .iter()
      .map(|(_, amount)| *amount)
      .collect::<Vec<_>>(),
    "each projectile's resolved damage must be applied in order"
  );
  assert_eq!(
    schedules.len(),
    5,
    "BFG 10K volley must schedule five delayed explosions"
  );
  assert!(
    schedules
      .iter()
      .all(|(_, delay, radius, knockback)| (*delay, *radius, *knockback) == (25, 2, 16)),
    "each BFG 10K schedule must preserve delay 25, radius 2, and knockback 16"
  );
  for (((attack_index, _), (damage_index, _)), (schedule_index, _, _, _)) in
    attacks.iter().zip(damages.iter()).zip(schedules.iter())
  {
    assert_eq!(
      *damage_index,
      *attack_index + 1,
      "each BFG 10K attack must be followed immediately by its damage event"
    );
    assert_eq!(
      *schedule_index,
      *damage_index + 1,
      "each BFG 10K damage event must be followed immediately by its schedule"
    );
  }
}

fn assert_standard_bfg_schedule_event(
  events: &[drl_protocol::GameEvent],
  attacker_id: drl_protocol::EntityId,
  target_id: drl_protocol::EntityId,
) {
  let attack_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        drl_protocol::GameEvent::AttackResolved {
          attacker_id: event_attacker,
          target_id: event_target,
          outcome: drl_protocol::AttackOutcome::Hit { .. },
          is_ranged: true,
        } if *event_attacker == attacker_id && *event_target == target_id
      )
    })
    .expect("standard BFG shot must resolve a hit");
  let damage_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        drl_protocol::GameEvent::DamageApplied { target_id: event_target, .. }
          if *event_target == target_id
      )
    })
    .expect("standard BFG shot must apply damage");
  let (schedule_index, delay, radius, knockback) = events
    .iter()
    .enumerate()
    .find_map(|(index, event)| match event {
      drl_protocol::GameEvent::Bfg9000ExplosionScheduled {
        entity_id,
        target_id: event_target,
        delay,
        radius,
        knockback,
      } if *entity_id == attacker_id && *event_target == target_id => {
        Some((index, *delay, *radius, *knockback))
      }
      _ => None,
    })
    .expect("standard BFG shot must schedule its delayed explosion");
  assert_eq!(damage_index, attack_index + 1);
  assert_eq!(schedule_index, damage_index + 1);
  assert_eq!((delay, radius, knockback), (33, 8, 16));
  assert_eq!(
    events
      .iter()
      .filter(|event| {
        matches!(
          event,
          drl_protocol::GameEvent::Bfg9000ExplosionScheduled { .. }
        )
      })
      .count(),
    1
  );
}

fn assert_nuclear_bfg_schedule_event(
  events: &[drl_protocol::GameEvent],
  attacker_id: drl_protocol::EntityId,
  target_id: drl_protocol::EntityId,
) {
  let attack_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        drl_protocol::GameEvent::AttackResolved {
          attacker_id: event_attacker,
          target_id: event_target,
          outcome: drl_protocol::AttackOutcome::Hit { .. },
          is_ranged: true,
        } if *event_attacker == attacker_id && *event_target == target_id
      )
    })
    .expect("Nuclear BFG shot must resolve a hit");
  let damage_index = events
    .iter()
    .position(|event| {
      matches!(
        event,
        drl_protocol::GameEvent::DamageApplied { target_id: event_target, .. }
          if *event_target == target_id
      )
    })
    .expect("Nuclear BFG shot must apply damage");
  let (schedule_index, delay, radius, knockback) = events
    .iter()
    .enumerate()
    .find_map(|(index, event)| match event {
      drl_protocol::GameEvent::NuclearBfg9000ExplosionScheduled {
        entity_id,
        target_id: event_target,
        delay,
        radius,
        knockback,
      } if *entity_id == attacker_id && *event_target == target_id => {
        Some((index, *delay, *radius, *knockback))
      }
      _ => None,
    })
    .expect("Nuclear BFG shot must schedule its delayed explosion");
  assert_eq!(damage_index, attack_index + 1);
  assert_eq!(schedule_index, damage_index + 1);
  assert_eq!((delay, radius, knockback), (33, 8, 16));
  assert_eq!(
    events
      .iter()
      .filter(|event| {
        matches!(
          event,
          drl_protocol::GameEvent::NuclearBfg9000ExplosionScheduled { .. }
        )
      })
      .count(),
    1
  );
}

fn test_item(name: &str) -> ItemView {
  ItemView {
    id: ItemId::new(7),
    archetype: ItemArchetype::Pistol,
    name: name.to_string(),
    category: ItemCategory::Weapon,
    count: 1,
    description: String::new(),
    clip: None,
    damage: None,
    armor_value: None,
    heal_amount: None,
    knockback: None,
    chainfire_level: 0,
  }
}
