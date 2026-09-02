use super::*;

#[test]
fn current_content_tables_pass_structural_validation() {
  assert_eq!(validate_current_content(), Ok(()));
}

#[test]
fn rejects_reversed_or_zero_damage_ranges() {
  assert!(matches!(
    require_range("monster", "test", "melee_damage", (0, 2)),
    Err(ContentValidationError::InvalidRange { .. })
  ));
  assert!(matches!(
    require_range("monster", "test", "melee_damage", (4, 2)),
    Err(ContentValidationError::InvalidRange { .. })
  ));
}

#[test]
fn rejects_invalid_roll_bound_order_and_coverage() {
  assert!(matches!(
    validate_roll_bounds("test-roll", &[40, 40, 100]),
    Err(ContentValidationError::InvalidRollBounds { index: 1, .. })
  ));
  assert_eq!(
    validate_roll_bounds("test-roll", &[40, 90]),
    Err(ContentValidationError::IncompleteRollBounds {
      table: "test-roll",
      final_bound: 90,
    })
  );
}

#[test]
fn rejects_invalid_level_dimensions_and_room_bounds() {
  let invalid_room = LevelDefinition {
    max_room_size: 4,
    min_room_size: 8,
    ..test_level()
  };
  assert!(matches!(
    validate_level_definition(invalid_room),
    Err(ContentValidationError::InvalidRange {
      field: "room_size",
      ..
    })
  ));

  let invalid_dimensions = LevelDefinition {
    width: 0,
    ..test_level()
  };
  assert_eq!(
    validate_level_definition(invalid_dimensions),
    Err(ContentValidationError::NonPositive {
      table: "level",
      key: "test",
      field: "width",
    })
  );
}

#[test]
fn rejects_ranged_weapon_without_range_or_ammo() {
  let invalid = ItemDefinition {
    kind: ItemDefinitionKind::Weapon {
      is_ranged: true,
      ammo_type: None,
      clip_capacity: 1,
      damage: (1, 2),
      range: 0,
      accuracy: 50,
      knockback: 0,
      fire_cost: drl_protocol::ActionCost::STANDARD,
      reload_cost: drl_protocol::ActionCost::STANDARD,
    },
    archetype: drl_protocol::ItemArchetype::Pistol,
    name: "test",
    description: "test",
  };
  assert_eq!(
    validate_item_definition(&invalid),
    Err(ContentValidationError::InvalidWeaponShape {
      key: "test",
      field: "ammo_type",
    })
  );
}

#[test]
fn rejects_ammo_initial_amount_above_stack_limit() {
  let invalid = ItemDefinition {
    kind: ItemDefinitionKind::Ammo {
      ammo_type: drl_protocol::AmmoType::Rocket,
      max_stack: 10,
      initial_amount: Some(11),
    },
    archetype: drl_protocol::ItemArchetype::AmmoRockets,
    name: "test rocket",
    description: "test",
  };
  assert_eq!(
    validate_item_definition(&invalid),
    Err(ContentValidationError::InvalidRange {
      table: "item",
      key: "test rocket",
      field: "initial_amount",
      minimum: 11,
      maximum: 10,
    })
  );
}

#[test]
fn rejects_ammo_pack_amount_above_capacity() {
  let invalid = ItemDefinition {
    kind: ItemDefinitionKind::AmmoPack {
      ammo_type: drl_protocol::AmmoType::Rocket,
      amount: 26,
      max_amount: 25,
    },
    archetype: drl_protocol::ItemArchetype::AmmoPackRockets,
    name: "test rocket box",
    description: "test",
  };
  assert_eq!(
    validate_item_definition(&invalid),
    Err(ContentValidationError::InvalidRange {
      table: "item",
      key: "test rocket box",
      field: "amount",
      minimum: 26,
      maximum: 25,
    })
  );
}

#[test]
fn rejects_resistance_above_one_hundred_percent() {
  let invalid = ItemDefinition {
    kind: ItemDefinitionKind::Armor {
      protection: 2,
      durability: 100,
      max_durability: 100,
      plasma_resistance: 101,
      fire_resistance: 0,
    },
    archetype: drl_protocol::ItemArchetype::BlueArmor,
    name: "test armor",
    description: "test",
  };
  assert_eq!(
    validate_item_definition(&invalid),
    Err(ContentValidationError::InvalidRange {
      table: "item",
      key: "test armor",
      field: "plasma_resistance",
      minimum: 101,
      maximum: 100,
    })
  );
}

#[test]
fn rejects_fire_resistance_above_one_hundred_percent() {
  let invalid = ItemDefinition {
    kind: ItemDefinitionKind::Armor {
      protection: 4,
      durability: 100,
      max_durability: 100,
      plasma_resistance: 0,
      fire_resistance: 101,
    },
    archetype: drl_protocol::ItemArchetype::RedArmor,
    name: "test red armor",
    description: "test",
  };
  assert_eq!(
    validate_item_definition(&invalid),
    Err(ContentValidationError::InvalidRange {
      table: "item",
      key: "test red armor",
      field: "fire_resistance",
      minimum: 101,
      maximum: 100,
    })
  );
}

fn test_level() -> LevelDefinition {
  LevelDefinition {
    key: "test",
    width: 10,
    height: 10,
    max_rooms: 1,
    min_room_size: 1,
    max_room_size: 1,
    max_monsters_per_room: 1,
    max_items_per_room: 1,
  }
}
