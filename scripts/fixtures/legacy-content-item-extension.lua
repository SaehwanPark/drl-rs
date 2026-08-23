register_item "chainsaw"
{
  name = "chainsaw",
  level = 4,
  type = "melee",
  OnUse = function(self, being)
    return true
  end,
}

register_item "bfg9000"
{
  name = "BFG 9000",
  level = 15,
  type = "ranged",
  damage = { min = 100, max = 200 },
}
