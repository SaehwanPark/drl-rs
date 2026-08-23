register_level "alpha"
{
  name = "Alpha level",
  level = 3,
  Create = function()
    return true
  end,
  map = [=[
{..}
register_level "embedded_fake"
]=],
}

register_level "beta"
{
  name = "Beta level",
  level = 9;
  canGenerate = function()
    return false
  end,
}

--[[
register_level "commented_out"
{
  level = 99,
}
]]
