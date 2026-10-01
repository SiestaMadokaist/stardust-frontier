# Architecture Plan

Status key: **[done]** exists in the repo today. **[planned]** discussed and agreed, not yet implemented.

## Project structure

Module files use the 2018+ file-module style (`screens.rs` + `screens/`), not `mod.rs`.

```
src/
  main.rs                    [done]  App wiring: DefaultPlugins, DataLoaderPlugin, state init, ScreensPlugin

  data.rs                    [done]  JSON-shaped definitions (serde DTOs)
  data/
    card.rs                  [done]  CardFile, CardDef, Cost, CostType, Target, Side, Multi, Effect
    character.rs              [done]  CharacterFile, Stats, Triggers

  loader.rs                  [done]  DataLoaderPlugin: scans ./loader/**/*.json, resolves each
                                      manifest's `root` + `paths`, populates CardDatabase /
                                      CharacterDatabase. Skips card files with "active": false.

  run.rs                     [planned]  RunState resource: picked character, hp, gold, map
                                         progress — data that survives screen transitions
  run/
    deck.rs                  [planned]  Deck: the run's full card pool (Vec<CardId> + per-card
                                         upgrades). Persists across battles.

  game.rs                    [planned]  GamePlugin: registers everything below. Screen-agnostic —
                                         no dependency on AppState or UI, testable standalone.
  game/
    card_instance.rs         [planned]  CardInstance component: card id + per-instance state
                                         (this-combat buffs, temp upgrades)
    piles.rs                 [planned]  Piles resource: draw/hand/discard/exhaust as Vec<Entity>.
                                         draw(n), discard_selected(ids), discard_random(n),
                                         peek_top(n)/peek_bottom(n), shuffle_discard_into_draw.
    effects.rs                [planned]  Generic effect interpreter: walks a CardDef's
                                         targets/effects and calls into piles/combatant state.
    handlers.rs               [planned]  Bespoke per-card handler registry — the escape hatch for
                                         cards whose effect can't be expressed generically
                                         (conditional/branching, scaling values, hand/deck
                                         manipulation, modal choices, cross-card references).
                                         Dispatches on card id; falls back to effects.rs when no
                                         handler is registered.

  save.rs                    [planned]  Serializes/deserializes SaveData to the platform save
                                         directory (via the `directories` crate), called at node-
                                         boundary points from screens/. See Checkpoint system below.

  screens.rs                 [planned]  ScreensPlugin, adds every screen plugin below
  screens/
    main_menu.rs              [planned]
    character_select.rs       [planned]
    stage_select.rs           [planned]  the map
    battle.rs                 [planned]  BattlePlugin + its own nested sub-state
    battle/
      setup.rs                [planned]  OnEnter(Battle): builds Piles + spawns CardInstance
                                         entities from run::Deck, shuffles draw
      player_turn.rs           [planned]  turn-start draw, play-card input, cost payment via
                                         game::piles, resolution via game::effects/handlers
      enemy_turn.rs             [planned]
      targeting.rs              [planned]
      ui.rs                     [planned]  renders hand/draw-count/discard-count, drag-and-drop
    reward.rs                 [planned]
    shop.rs                   [planned]
    rest.rs                   [planned]
    game_over.rs               [planned]

  ui.rs                      [planned]  shared widgets reused across screens (buttons, card-render
  ui/                                   widget, HP bar)
    widgets.rs                [planned]

data/                        [done]  shipped game content (JSON)
  cards/*.json
  characters/*.json
  images/

loader/                      [done]  manifests: {"root": "...", "entries": [{"kind", "paths"}]}
mods/                        [done, empty]  mods drop their own manifest JSON into ./loader
                                            and keep their own data under mods/<name>/
```

### Key structural decisions

- **`data/` vs `game/`** — `data/` holds pure JSON-shaped DTOs (what a card's *definition* says).
  `game/` holds the screen-agnostic mechanics that interpret those DTOs at runtime (what actually
  happens when the card is played). Keeps the mechanics testable without spinning up any UI/state
  machine, the same way the current loader test runs today.
- **`run/` vs `game/`** — `run::Deck` is "what cards do I own" (survives the whole run, across
  battles). `game::Piles` is "where are my cards right now" (draw/hand/discard/exhaust), rebuilt
  from `run::Deck` at the start of every battle and discarded at the end.
- **Card instances are entities, not plain ids** — `CardInstance` is an ECS component so hand/draw/
  discard entities can also carry visual components (`Transform`, sprite, drag interaction) without
  a second parallel representation.
- **Generic effects + bespoke handler escape hatch** — most cards (flat damage/heal/burn/shield to
  N selected/random/all targets) resolve through the generic `effects.rs` interpreter driven purely
  by JSON. Cards needing branching logic, scaling values, hand/deck manipulation, modal choices, or
  cross-card references register a handler in `handlers.rs`, keyed by card id, with `effects.rs` as
  the fallback.

## Checkpoint system (save/resume across process restarts)

**[planned]**

- **What's saved**: a `SaveData` snapshot distinct from the live `RunState`/`Deck` — character,
  owned card ids + per-card upgrades, hp, gold, relics, map seed + current node.
- **When it's saved**: at map-node-completion boundaries (end of battle/shop/rest), not mid-combat.
  Mid-fight state (`Piles`, live `Entity` handles) is never serialized — those aren't trivially
  serializable without an id-remapping layer, and no save is needed for something recoverable by
  just re-fighting from the node start (see Rollback below).
- **Where it's saved**: the platform's actual save-data directory via the `directories` crate
  (`%APPDATA%\stardust-frontiers\saves\` on Windows), not next to `./data`/`./loader` — that tree
  may be read-only (Program Files install) or get overwritten by an update/mod reinstall.
- **Format**: JSON via serde, consistent with the rest of the project's data files. Chosen over
  SQLite because a single run snapshot is naturally nested (deck as a list of ids + per-card state,
  map as a tree) and evolves painlessly during dev — `#[serde(default)]` absorbs new fields without
  a migration. SQLite would only earn its place if meta-progression across *many* runs (stats,
  unlocks, achievements, run history) becomes a feature — that's genuinely relational/queryable
  data, distinct from "the current run's snapshot," and could coexist with it later.
- **Mod safety**: saves store card/character **ids** (strings), not inline data. Loading a save
  must tolerate a referenced id that no longer exists (mod removed or renamed) rather than
  hard-crash — otherwise an abandoned mod bricks old saves.

## Rollback / crash-recovery

**[planned]** Two distinct concerns, with different mechanisms:

1. **"The program closed mid-battle — where do we resume?"**
   Answer: nowhere special. Battle state (`Piles`, combatant hp/energy/status) lives purely in
   memory for the fight's duration and is never written to disk. On relaunch, the player resumes
   from the last checkpoint — start of the current node/battle — from the `SaveData` above. This
   avoids the real complexity of serializing live `Entity` state mid-combat, for a case the genre
   doesn't typically need finer-grained recovery for anyway.

2. **In-fight undo** (optional gameplay mechanic, not crash recovery): a `Vec<BattleSnapshot>`
   pushed each step (card played, turn ended), purely in-memory, popped to undo. Cheap because
   `Piles` + combatant state is small. This stack is never persisted — closing the game loses it,
   same as any other mid-battle state, which is consistent with (1) above.

SQLite does not change either of these: rollback is an in-memory history stack (snapshots or
reversible commands) regardless of what eventually backs on-disk storage. A database would only
matter if the in-fight undo stack itself needed to survive a restart — a stronger requirement than
either concern above currently calls for.
