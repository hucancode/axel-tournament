# Bracket Protocol

How tournament brackets get generated and advanced. Lives in
`api/src/services/bracket.rs` and is driven by `healer::tick`.

Two shapes are supported:

| `match_generation_type` | Meaning                                  |
|-------------------------|------------------------------------------|
| `single_elimination`    | Standard knockout. One loss → out.       |
| `double_elimination`    | Two losses → out. Has a winners bracket, |
|                         | a losers bracket, a grand final, and an  |
|                         | optional grand-final reset.              |

Round-robin and all-vs-all are handled elsewhere and are not brackets.

---

## Match shape

Every match row carries the bracket coordinates needed to advance:

```text
match {
  round            : u32     // 0 = first round
  bracket          : "winners" | "losers" | "grand_final" | "grand_final_reset"
  bracket_position : u32     // index within (round, bracket)
  participants     : [...]   // 1 entry = BYE, 2 entries = real match
  status           : pending | running | completed | failed
  faulted_user_ids : [user]  // bots that crashed / illegal-moved
}
```

Position 0 is the top of the bracket. Within a round, positions are
laid out left-to-right.

---

## Round-zero seeding (single elim)

`single_elim_round_zero(n)` returns the round-0 pairings using
*1-vs-N* seeding (highest seed plays lowest, then innermost outwards).
Inputs are zero-indexed seed numbers.

* `n = 2` → `[(0, Some(1))]`
* `n = 8` → `[(0, 7), (3, 4), (1, 6), (2, 5)]`
* Non-power-of-2 `n` → bracket pads up to the next power of two and
  the top seeds get `None` (a BYE). E.g. `n = 3` → `[(0, None),
  (1, Some(2))]`.

A BYE match is created in the `pending` state with one participant and
auto-completed as soon as it is read; its sole player advances.

---

## Single-elimination advancement

After every match write the healer calls `advance_brackets` which calls
`advance_single`. The rule:

1. Find `max_round` of `winners`-bracket matches.
2. If `max_round + 1` already has matches → done.
3. If any match in `max_round` is still pending → wait.
4. Otherwise, sort the round's matches by `bracket_position` and pair
   them: positions `(2k, 2k+1)` create round `max_round + 1`,
   position `k`. The winner of each parent match is the participant.
5. When `max_round` has only one match and it is terminal, the bracket
   is over. Finalization is handled by `finalize_if_done`, not
   `advance_brackets`.

---

## Double-elimination layout

For a power-of-two bracket of `N = 2^k` players:

```text
Winners bracket : WB R0 .. WB R(k-1)        // standard single-elim
Losers bracket  : LB R0 .. LB R(2k-3)       // 2(k-1) rounds total
Grand final     : winner(WB) vs winner(LB)  // bracket = "grand_final"
Grand final
  reset         : only created if LB-side won the GF
                  (bracket = "grand_final_reset")
```

Losers-bracket round indexing alternates between *drop* rounds (a new
batch of WB losers joins) and *internal* rounds (the LB plays itself):

| LB round   | Feeders                                   |
|------------|-------------------------------------------|
| `R0`       | WB R0 losers paired with each other       |
| `R(2j+1)`  | LB R(2j) winners + WB R(j+1) losers       |
| `R(2j+2)`  | LB R(2j+1) winners paired with each other |

`advance_losers` walks LB rounds in order. For each round it:

1. Skips if the round already has matches.
2. Asks `lb_round_pairs` for the feeder participants. If any feeder
   match is still pending, returns `None` and the loop stops — we
   come back next tick.
3. Otherwise creates the round's matches at consecutive `bracket_position`
   values starting at 0.

---

## Grand final and reset

`advance_grand_final` runs after WB and LB advancement:

1. If no `grand_final` match exists yet, look up the WB final winner
   and the LB final winner. As soon as both exist, create the
   grand-final match with `round = 0`, `bracket = "grand_final"`,
   participants `[wb_winner, lb_winner]` (WB-side at index 0).
2. If the grand final is terminal:
   * WB-side won → bracket is finished. No reset.
   * LB-side won → create `grand_final_reset` with the same two
     players. Whoever wins the reset takes the tournament.

---

## Determining a match winner

`winner_of(match)` is the single source of truth for advancement:

* `Completed` match → the participant with the highest `score` who is
  *not* in `faulted_user_ids`. Faulted bots cannot advance even if
  their score happens to be highest.
* `Failed` match → if exactly one non-faulted participant remains,
  they advance. If both sides faulted, no winner is produced and the
  bracket short-circuits via `finalize_if_done`.
* Anything else → `None` (match is still in progress).

This is why a runtime error or illegal move counts as a *bot-only*
loss: the faulted side is removed from the candidate set, and their
opponent advances cleanly.

---

## Concurrency and idempotency

`advance_brackets` is safe to call repeatedly. Each step checks for
already-existing next-round matches before creating any. The healer
loop calls it on every tick for every running tournament; a duplicate
call is a no-op.

When two healer ticks race on the same tournament, the worst case is
that both observe "round N+1 not yet created" simultaneously and both
attempt to insert. The match table has no unique constraint on
`(tournament_id, round, bracket, bracket_position)` today, so this is
the bracket's known weak point — single healer instance per database
is the deployment assumption.
