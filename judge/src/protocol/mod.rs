// Wire parser/serializer. Spec: judge/protocols/wire.md.

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientFrame {
    Hello { jwt: String, since: u64 },
    Act { kind: String, payload: String },
    Pong,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerFrame {
    Welcome { player_id: String, head: u64 },
    Event { seq: u64, kind: String, payload: String },
    Err { code: String, msg: String },
    Ping,
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("empty frame")]
    Empty,
    #[error("unknown verb: {0}")]
    UnknownVerb(String),
    #[error("malformed {verb}: {reason}")]
    Malformed { verb: &'static str, reason: String },
}

pub fn parse_client(line: &str) -> Result<ClientFrame, ParseError> {
    let line = line.trim();
    if line.is_empty() {
        return Err(ParseError::Empty);
    }

    let (verb, rest) = split_first(line);
    match verb {
        "HELLO" => {
            let (jwt, rest) = split_first(rest);
            let since_tok = rest.trim();
            if jwt.is_empty() || since_tok.is_empty() {
                return Err(ParseError::Malformed {
                    verb: "HELLO",
                    reason: "expected: HELLO <jwt> <since_seq>".into(),
                });
            }
            let since: u64 = since_tok.parse().map_err(|_| ParseError::Malformed {
                verb: "HELLO",
                reason: format!("since not a u64: {since_tok}"),
            })?;
            Ok(ClientFrame::Hello {
                jwt: jwt.to_string(),
                since,
            })
        }
        "ACT" => {
            let (kind, payload) = split_first(rest);
            if kind.is_empty() {
                return Err(ParseError::Malformed {
                    verb: "ACT",
                    reason: "expected: ACT <kind> [payload]".into(),
                });
            }
            Ok(ClientFrame::Act {
                kind: kind.to_string(),
                payload: payload.to_string(),
            })
        }
        "PONG" => Ok(ClientFrame::Pong),
        other => Err(ParseError::UnknownVerb(other.to_string())),
    }
}

pub fn serialize_client(frame: &ClientFrame) -> String {
    match frame {
        ClientFrame::Hello { jwt, since } => format!("HELLO {jwt} {since}"),
        ClientFrame::Act { kind, payload } => {
            if payload.is_empty() {
                format!("ACT {kind}")
            } else {
                format!("ACT {kind} {payload}")
            }
        }
        ClientFrame::Pong => "PONG".to_string(),
    }
}

pub fn serialize_server(frame: &ServerFrame) -> String {
    match frame {
        ServerFrame::Welcome { player_id, head } => format!("WELCOME {player_id} {head}"),
        ServerFrame::Event { seq, kind, payload } => {
            if payload.is_empty() {
                format!("EVENT {seq} {kind}")
            } else {
                format!("EVENT {seq} {kind} {payload}")
            }
        }
        ServerFrame::Err { code, msg } => format!("ERR {code} {msg}"),
        ServerFrame::Ping => "PING".to_string(),
    }
}

pub fn parse_server(line: &str) -> Result<ServerFrame, ParseError> {
    let line = line.trim();
    if line.is_empty() {
        return Err(ParseError::Empty);
    }
    let (verb, rest) = split_first(line);
    match verb {
        "WELCOME" => {
            let (pid, rest) = split_first(rest);
            let head_tok = rest.trim();
            if pid.is_empty() || head_tok.is_empty() {
                return Err(ParseError::Malformed {
                    verb: "WELCOME",
                    reason: "expected: WELCOME <player_id> <head_seq>".into(),
                });
            }
            let head: u64 = head_tok.parse().map_err(|_| ParseError::Malformed {
                verb: "WELCOME",
                reason: format!("head not a u64: {head_tok}"),
            })?;
            Ok(ServerFrame::Welcome {
                player_id: pid.to_string(),
                head,
            })
        }
        "EVENT" => {
            let (seq_tok, rest) = split_first(rest);
            let (kind, payload) = split_first(rest);
            if seq_tok.is_empty() || kind.is_empty() {
                return Err(ParseError::Malformed {
                    verb: "EVENT",
                    reason: "expected: EVENT <seq> <kind> [payload]".into(),
                });
            }
            let seq: u64 = seq_tok.parse().map_err(|_| ParseError::Malformed {
                verb: "EVENT",
                reason: format!("seq not a u64: {seq_tok}"),
            })?;
            Ok(ServerFrame::Event {
                seq,
                kind: kind.to_string(),
                payload: payload.to_string(),
            })
        }
        "ERR" => {
            let (code, msg) = split_first(rest);
            if code.is_empty() {
                return Err(ParseError::Malformed {
                    verb: "ERR",
                    reason: "expected: ERR <code> <msg>".into(),
                });
            }
            Ok(ServerFrame::Err {
                code: code.to_string(),
                msg: msg.to_string(),
            })
        }
        "PING" => Ok(ServerFrame::Ping),
        other => Err(ParseError::UnknownVerb(other.to_string())),
    }
}

fn split_first(s: &str) -> (&str, &str) {
    let s = s.trim_start();
    match s.find(char::is_whitespace) {
        Some(i) => (&s[..i], s[i..].trim_start()),
        None => (s, ""),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hello() {
        let f = parse_client("HELLO abc.def.ghi 17").unwrap();
        assert_eq!(
            f,
            ClientFrame::Hello {
                jwt: "abc.def.ghi".into(),
                since: 17,
            }
        );
    }

    #[test]
    fn parses_act_with_spaces_in_payload() {
        let f = parse_client("ACT CHAT hello world  with spaces").unwrap();
        assert_eq!(
            f,
            ClientFrame::Act {
                kind: "CHAT".into(),
                payload: "hello world  with spaces".into(),
            }
        );
    }

    #[test]
    fn parses_act_no_payload() {
        let f = parse_client("ACT START").unwrap();
        assert_eq!(
            f,
            ClientFrame::Act {
                kind: "START".into(),
                payload: "".into(),
            }
        );
    }

    #[test]
    fn parses_pong() {
        assert_eq!(parse_client("PONG").unwrap(), ClientFrame::Pong);
    }

    #[test]
    fn rejects_unknown_verb() {
        assert!(matches!(parse_client("FOO bar"), Err(ParseError::UnknownVerb(_))));
    }

    #[test]
    fn rejects_bad_since() {
        assert!(matches!(
            parse_client("HELLO j notanum"),
            Err(ParseError::Malformed { .. })
        ));
    }

    #[test]
    fn serializes_event() {
        let s = serialize_server(&ServerFrame::Event {
            seq: 7,
            kind: "MOVE".into(),
            payload: "1 1".into(),
        });
        assert_eq!(s, "EVENT 7 MOVE 1 1");
    }

    #[test]
    fn serializes_event_no_payload() {
        let s = serialize_server(&ServerFrame::Event {
            seq: 1,
            kind: "GAME_STARTED".into(),
            payload: "".into(),
        });
        assert_eq!(s, "EVENT 1 GAME_STARTED");
    }

    #[test]
    fn serializes_welcome() {
        let s = serialize_server(&ServerFrame::Welcome {
            player_id: "user:xyz".into(),
            head: 42,
        });
        assert_eq!(s, "WELCOME user:xyz 42");
    }

    #[test]
    fn rejects_empty_frame() {
        assert!(matches!(parse_client(""), Err(ParseError::Empty)));
        assert!(matches!(parse_client("   "), Err(ParseError::Empty)));
        assert!(matches!(parse_client("\n\t  "), Err(ParseError::Empty)));
    }

    #[test]
    fn hello_missing_fields() {
        assert!(matches!(
            parse_client("HELLO"),
            Err(ParseError::Malformed { verb: "HELLO", .. })
        ));
        assert!(matches!(
            parse_client("HELLO jwt"),
            Err(ParseError::Malformed { verb: "HELLO", .. })
        ));
    }

    #[test]
    fn hello_rejects_negative_since() {
        assert!(matches!(
            parse_client("HELLO j -1"),
            Err(ParseError::Malformed { verb: "HELLO", .. })
        ));
    }

    #[test]
    fn hello_accepts_zero_since() {
        let f = parse_client("HELLO j 0").unwrap();
        assert_eq!(
            f,
            ClientFrame::Hello {
                jwt: "j".into(),
                since: 0,
            }
        );
    }

    #[test]
    fn hello_tolerates_extra_whitespace() {
        let f = parse_client("  HELLO   abc.def.ghi   17  ").unwrap();
        assert_eq!(
            f,
            ClientFrame::Hello {
                jwt: "abc.def.ghi".into(),
                since: 17,
            }
        );
    }

    #[test]
    fn hello_rejects_overflow_since() {
        assert!(matches!(
            parse_client("HELLO j 18446744073709551616"),
            Err(ParseError::Malformed { verb: "HELLO", .. })
        ));
    }

    #[test]
    fn act_missing_kind() {
        assert!(matches!(
            parse_client("ACT"),
            Err(ParseError::Malformed { verb: "ACT", .. })
        ));
        assert!(matches!(
            parse_client("ACT   "),
            Err(ParseError::Malformed { verb: "ACT", .. })
        ));
    }

    #[test]
    fn act_payload_preserves_internal_spaces() {
        let f = parse_client("ACT MOVE 1 1").unwrap();
        assert_eq!(
            f,
            ClientFrame::Act {
                kind: "MOVE".into(),
                payload: "1 1".into(),
            }
        );
    }

    #[test]
    fn act_payload_preserves_unicode() {
        let f = parse_client("ACT CHAT héllo 🎮 wörld").unwrap();
        assert_eq!(
            f,
            ClientFrame::Act {
                kind: "CHAT".into(),
                payload: "héllo 🎮 wörld".into(),
            }
        );
    }

    #[test]
    fn pong_with_trailing_whitespace() {
        assert_eq!(parse_client("PONG  ").unwrap(), ClientFrame::Pong);
        assert_eq!(parse_client("  PONG").unwrap(), ClientFrame::Pong);
    }

    #[test]
    fn unknown_verb_carries_token() {
        match parse_client("YO mate") {
            Err(ParseError::UnknownVerb(v)) => assert_eq!(v, "YO"),
            other => panic!("expected UnknownVerb, got {other:?}"),
        }
    }

    #[test]
    fn parse_is_case_sensitive() {
        assert!(matches!(
            parse_client("hello j 0"),
            Err(ParseError::UnknownVerb(_))
        ));
        assert!(matches!(
            parse_client("Act MOVE"),
            Err(ParseError::UnknownVerb(_))
        ));
    }

    #[test]
    fn serializes_err() {
        let s = serialize_server(&ServerFrame::Err {
            code: "AUTH".into(),
            msg: "bad token".into(),
        });
        assert_eq!(s, "ERR AUTH bad token");
    }

    #[test]
    fn serializes_ping() {
        assert_eq!(serialize_server(&ServerFrame::Ping), "PING");
    }

    #[test]
    fn serializes_event_with_multispace_payload() {
        let s = serialize_server(&ServerFrame::Event {
            seq: 9,
            kind: "CHAT".into(),
            payload: "user:alice  hello   world".into(),
        });
        assert_eq!(s, "EVENT 9 CHAT user:alice  hello   world");
    }

    #[test]
    fn frames_have_no_trailing_newline() {
        for frame in [
            ServerFrame::Welcome { player_id: "u".into(), head: 0 },
            ServerFrame::Event { seq: 1, kind: "K".into(), payload: "".into() },
            ServerFrame::Err { code: "X".into(), msg: "y".into() },
            ServerFrame::Ping,
        ] {
            let s = serialize_server(&frame);
            assert!(!s.ends_with('\n'), "frame must not end with newline: {s:?}");
            assert!(!s.contains('\n'), "frame must be single-line: {s:?}");
        }
    }

    #[test]
    fn round_trip_client_frames() {
        for f in [
            ClientFrame::Hello { jwt: "abc".into(), since: 0 },
            ClientFrame::Hello { jwt: "abc.def".into(), since: 17 },
            ClientFrame::Act { kind: "JOIN".into(), payload: "".into() },
            ClientFrame::Act { kind: "MOVE".into(), payload: "1 2".into() },
            ClientFrame::Act { kind: "CHAT".into(), payload: "hello world".into() },
            ClientFrame::Pong,
        ] {
            let line = serialize_client(&f);
            let back = parse_client(&line).unwrap();
            assert_eq!(back, f);
        }
    }

    #[test]
    fn round_trip_server_frames() {
        for f in [
            ServerFrame::Welcome { player_id: "user:alice".into(), head: 0 },
            ServerFrame::Welcome { player_id: "user:alice".into(), head: 99 },
            ServerFrame::Event { seq: 1, kind: "GAME_STARTED".into(), payload: "".into() },
            ServerFrame::Event { seq: 2, kind: "MOVE".into(), payload: "user:alice ROCK".into() },
            ServerFrame::Event {
                seq: 3,
                kind: "ROUND_RESULT".into(),
                payload: "1 ROCK PAPER 0 1".into(),
            },
            ServerFrame::Err { code: "AUTH".into(), msg: "bad token".into() },
            ServerFrame::Ping,
        ] {
            let line = serialize_server(&f);
            let back = parse_server(&line).unwrap();
            assert_eq!(back, f);
        }
    }
}
