function drl.register_beings()
  register_being "imp"
  {
    name = "imp",
    hp = 12,
    speed = 105,
    corpse = true,
    desc = "literal } brace",
    flags = { BF_OPENDOORS },
    OnCreate = function (self)
      self.eq.weapon = "fireball"
    end,
  }

  register_being "former"
  {
    name = "former human",
    hp = 10,
    speed = 90,
    corpse = true,
  }

  register_item "garmor"
  {
    name = "green armor",
    level = 1,
    armor = 1,
    type = "armor",
    resist = { bullet = 15 },
  }
end
