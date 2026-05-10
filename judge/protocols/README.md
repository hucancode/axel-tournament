# Room Protocol

Event-sourced room protocol used by the judge for human (and mixed
human + bot) matches.

The whole protocol fits in three layers. Each layer is replaceable on
its own. Each can be tested on its own.

```
┌──────────────────────────────────────────────────────────────────┐
│  Wire     — frame syntax (parse / serialize, no state)           │
├──────────────────────────────────────────────────────────────────┤
│  Logic    — RoomLogic: pure validate + fold (no IO)              │
├──────────────────────────────────────────────────────────────────┤
│  Storage  — LeaseStore + EventLog + MetaIndex (the only IO)      │
└──────────────────────────────────────────────────────────────────┘
```

The protocol is one document set:

1. [`wire.md`](wire.md) — frame format. Seven verbs, UTF-8 lines.
2. [`architecture.md`](architecture.md) — `RoomLogic`, the three
   storage traits, the `LiveRoom` runtime, lease/failover, and the
   DB-free test recipe.
3. Per-game contracts — the `kind` strings and what each one means:
   - [`rock-paper-scissors.md`](rock-paper-scissors.md)
   - [`tic-tac-toe.md`](tic-tac-toe.md)
   - [`prisoners-dilemma.md`](prisoners-dilemma.md)
4. [`brackets.md`](brackets.md) — single + double elimination bracket
   generation, advancement, and grand-final reset rules.

## Design rules

- **One source of truth per room.** The append-only event log. State
  is a pure fold of the log. The runtime carries no fact the log does
  not.
- **One wire format.** Same seven frames for every game. Per-game
  semantics live entirely in the `kind` string and its payload — the
  wire never grows new verbs to add a game.
- **Pure logic, IO at the edges.** `RoomLogic` is no-IO. Persistence
  sits behind three small traits (`LeaseStore`, `EventLog`,
  `MetaIndex`). The runtime glues them.
- **Testable without a database.** Every storage trait has an
  in-memory implementation. A full `validate → append → fold →
  broadcast` test runs against `MemoryStorage` under
  `#[tokio::test]` — no DB, no migrations, no globals.
- **No hidden translations.** The client renders the same events the
  server appended. There is no second per-game text protocol.

If a change cannot be expressed inside these rules, the rules are
wrong — not the change. Update the affected document together with
the code.
