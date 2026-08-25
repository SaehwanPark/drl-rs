# Vertical Subtle Knife Encounter — Test Plan

1. Build a declarative ASCII scenario with a configured Subtle Knife, a visible
   Imp, an occluding wall, and a hidden Imp.
2. Run `Command::Invoke(ItemId::new(4))` through `ScenarioRunner`; assert the
   target list, player cost, event ordering, hidden-target exclusion, and
   deterministic replay verification.
3. Construct the same initial replay game at the browser boundary; compare
   `BrowserSession::submit` with direct `Game::step` for events, observations,
   effects, and scene state.
4. Run focused core/web tests, the workspace suite, repository/version checks,
   and diff whitespace validation.
