# Wire format

UTF-8 line-equivalent frames. One frame per logical message. Tokens
are separated by ASCII spaces; only the trailing payload may contain
spaces. Frames must not contain `\n` — line terminators are
transport-supplied.

## Transports

The same seven frames travel over two transports:

1. **WebSocket** (humans). One frame per `ws.send`. The server is
   `ws://<judge>/ws/<game_id>/<room_id>`; the session begins with
   `HELLO <jwt> <since_seq>` and supports reconnect via `since_seq`.
2. **Stdio** (bots). One newline-terminated frame per line on the
   subprocess stdin (server→bot) or stdout (bot→server). The judge
   that spawned the bot has already authorized it, so the bot
   transport drops `HELLO`, `WELCOME`, `since_seq`, `PING`, `PONG` —
   bots cannot reconnect, they are torn down on disconnect. Everything
   else (`ACT`, `EVENT`, `ERR`) is identical.

The pure parser/serializer in `src/protocol/mod.rs` is shared by both.

## HTTP endpoints (WebSocket transport)

```
ws://<judge>/ws/<game_id>/<room_id>          room session (auth via HELLO)
GET <judge>/api/rooms?game=&phase=&limit=    discovery (anonymous)
GET <judge>/health                           liveness
GET <judge>/capacity                         AI-vs-AI capacity (unrelated)
```

`game_id` and `room_id` are bound by the URL. The server uses them as
the authority for the rest of the session — the client does not repeat
them in any frame.

## Frames

There are exactly seven verbs. Every game uses the same seven; only
the `kind` strings inside `EVENT` / `ACT` differ.

### Client → server

```
HELLO <jwt> <since_seq>
ACT   <kind> [payload...]
PONG
```

- `HELLO` is the **only** valid first frame. `since_seq` is the
  highest `EVENT seq` the client has already applied (`0` for a
  fresh connection).
- `ACT` requests a state change. `<kind>` is per-game; `[payload]` is
  a single trailing string that may contain spaces.
- `PONG` answers the server's `PING`.

### Server → client

```
WELCOME <player_id> <head_seq>
EVENT   <seq> <kind> [payload...]
ERR     <code> <msg...>
PING
```

- `WELCOME` follows a successful `HELLO`. `player_id` is the
  authenticated user; `head_seq` is the current log head at the
  moment the server subscribed the client to the live stream.
- `EVENT` is the only state-bearing frame. Sequence numbers are
  monotonically increasing per room and never skip.
- `ERR` reports a protocol, auth, or server error. The connection
  may close after.
- `PING` is sent every 20 s; the client must reply `PONG`.

## Connect and reconnect

The same procedure handles both. The client opens the WebSocket,
sends `HELLO`, applies the events the server gap-fills, then
continues live.

```
C: HELLO <jwt> 0                   ← fresh connect
S: WELCOME user:alice 0
S: PING
... live stream ...
```

```
C: HELLO <jwt> 17                  ← reconnect, last seq seen was 17
S: WELCOME user:alice 42
S: EVENT 18 MOVE user:alice ROCK
S: EVENT 19 MOVE user:bob   PAPER
S: EVENT 20 ROUND_RESULT 1 ROCK PAPER 0 1
... live stream resumes ...
```

The server gap-fills `since_seq+1..head_seq` after `WELCOME`, then
multiplexes any further appends. The client's only obligation is to
keep its `since_seq` cursor up to date.

## Errors and closure

`ERR <code> <msg>` carries a short upper-snake-case code and a
free-form message. Standard codes:

| code             | meaning                                                |
|------------------|--------------------------------------------------------|
| `PARSE`          | malformed frame                                        |
| `EXPECTED_HELLO` | first frame was not `HELLO`                            |
| `AUTH`           | JWT missing / expired / invalid                        |
| `TIMEOUT`        | `HELLO` not received within the grace window           |
| `ROOM_OPEN`      | failed to acquire lease / load room                    |
| `READ`           | gap-fill read against the event log failed             |

Action validation failures are silent on the wire: if
`RoomLogic::validate` rejects an `ACT`, the server simply does not
append. Clients infer the outcome from the absence of the expected
`EVENT`. This keeps the wire strictly authoritative — clients only
ever see committed events.

## Encoding

- Frames are line-equivalent UTF-8. They MUST NOT contain `\n`.
  Internal whitespace inside a payload is preserved verbatim.
- Verbs are uppercase ASCII (`HELLO`, `ACT`, …). Case-sensitive.
- `seq`, `since_seq`, `head_seq` are decimal `u64`.
- `<jwt>`, `<player_id>`, `<kind>`, `<code>` contain no whitespace.

A reference parser / serializer lives in `src/protocol/mod.rs`. Its
unit tests cover the round-trips this document requires.
