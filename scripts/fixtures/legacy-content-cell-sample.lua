function drl.register_cells()
  register_cell "wall"
  {
    name = "base wall",
    ascii = "#",
    asciilow = '.',
    hp = 10,
    flags = { CF_BLOCKLOS },
    OnAct = function(c, being)
      return true
    end,
  }

  register_cell "floor"
  {
    name = "floor",
    ascii = "\250",
    set = CELLSET_FLOORS,
    bloodto = "bloodpool";
  }
end
