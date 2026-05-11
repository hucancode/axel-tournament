use crate::{
    error::{ApiError, ApiResult},
    models::{ProgrammingLanguage, find_game_by_id},
};
use axum::{Json, extract::Path};
use serde::Serialize;

// Reference bots that already ship as judge test fixtures: embed them
// at compile time so the API can serve language-specific starter code
// without runtime filesystem access.
const RPS_ROCK_RS: &str = include_str!("../../../judge/tests/bots/rps_rock.rs");
const RPS_ROCK_GO: &str = include_str!("../../../judge/tests/bots/rps_rock.go");
const RPS_ROCK_C: &str = include_str!("../../../judge/tests/bots/rps_rock.c");
const PD_COOP_RS: &str = include_str!("../../../judge/tests/bots/pd_cooperate.rs");
const TTT_TOP_RS: &str = include_str!("../../../judge/tests/bots/ttt_top_row.rs");

// Starter templates for combos without a matching test fixture. Some
// are working ports (PD/TTT in Go/C, Jar-of-Greed); chess/xiangqi/poker
// ship as documented skeletons because a useful default would require
// a full engine.
const PD_COOP_GO: &str = include_str!("starter_templates/pd_cooperate.go");
const PD_COOP_C: &str = include_str!("starter_templates/pd_cooperate.c");
const TTT_TOP_GO: &str = include_str!("starter_templates/ttt_top_row.go");
const TTT_TOP_C: &str = include_str!("starter_templates/ttt_top_row.c");
const JOG_RS: &str = include_str!("starter_templates/jog_contribute.rs");
const JOG_GO: &str = include_str!("starter_templates/jog_contribute.go");
const JOG_C: &str = include_str!("starter_templates/jog_contribute.c");
const CHESS_RS: &str = include_str!("starter_templates/chess_skeleton.rs");
const CHESS_GO: &str = include_str!("starter_templates/chess_skeleton.go");
const CHESS_C: &str = include_str!("starter_templates/chess_skeleton.c");
const XIANGQI_RS: &str = include_str!("starter_templates/xiangqi_skeleton.rs");
const XIANGQI_GO: &str = include_str!("starter_templates/xiangqi_skeleton.go");
const XIANGQI_C: &str = include_str!("starter_templates/xiangqi_skeleton.c");
const POKER_RS: &str = include_str!("starter_templates/poker_skeleton.rs");
const POKER_GO: &str = include_str!("starter_templates/poker_skeleton.go");
const POKER_C: &str = include_str!("starter_templates/poker_skeleton.c");

#[derive(Debug, Clone, Serialize)]
pub struct StarterCodeResponse {
    pub game_id: String,
    pub language: String,
    pub filename: String,
    pub code: String,
}

/// Resolve a per-game / per-language starter bot for the submission
/// editor. Every supported (game, language) combination returns a
/// template — working reference bots where they exist, documented
/// skeletons otherwise.
pub async fn get_starter_code(
    Path((game_id, language)): Path<(String, String)>,
) -> ApiResult<Json<StarterCodeResponse>> {
    let game = find_game_by_id(&game_id)
        .ok_or_else(|| ApiError::NotFound("Game not found".to_string()))?;
    let lang = ProgrammingLanguage::from_str(&language)
        .ok_or_else(|| ApiError::BadRequest(format!("Unsupported language: {language}")))?;

    let (code, filename) = match (game.id, &lang) {
        ("rock-paper-scissors", ProgrammingLanguage::Rust) => (RPS_ROCK_RS, "rps_rock.rs"),
        ("rock-paper-scissors", ProgrammingLanguage::Go) => (RPS_ROCK_GO, "rps_rock.go"),
        ("rock-paper-scissors", ProgrammingLanguage::C) => (RPS_ROCK_C, "rps_rock.c"),

        ("prisoners-dilemma", ProgrammingLanguage::Rust) => (PD_COOP_RS, "pd_cooperate.rs"),
        ("prisoners-dilemma", ProgrammingLanguage::Go) => (PD_COOP_GO, "pd_cooperate.go"),
        ("prisoners-dilemma", ProgrammingLanguage::C) => (PD_COOP_C, "pd_cooperate.c"),

        ("tic-tac-toe", ProgrammingLanguage::Rust) => (TTT_TOP_RS, "ttt_top_row.rs"),
        ("tic-tac-toe", ProgrammingLanguage::Go) => (TTT_TOP_GO, "ttt_top_row.go"),
        ("tic-tac-toe", ProgrammingLanguage::C) => (TTT_TOP_C, "ttt_top_row.c"),

        ("jar-of-greed", ProgrammingLanguage::Rust) => (JOG_RS, "jog_contribute.rs"),
        ("jar-of-greed", ProgrammingLanguage::Go) => (JOG_GO, "jog_contribute.go"),
        ("jar-of-greed", ProgrammingLanguage::C) => (JOG_C, "jog_contribute.c"),

        ("chess", ProgrammingLanguage::Rust) => (CHESS_RS, "chess_skeleton.rs"),
        ("chess", ProgrammingLanguage::Go) => (CHESS_GO, "chess_skeleton.go"),
        ("chess", ProgrammingLanguage::C) => (CHESS_C, "chess_skeleton.c"),

        ("xiangqi", ProgrammingLanguage::Rust) => (XIANGQI_RS, "xiangqi_skeleton.rs"),
        ("xiangqi", ProgrammingLanguage::Go) => (XIANGQI_GO, "xiangqi_skeleton.go"),
        ("xiangqi", ProgrammingLanguage::C) => (XIANGQI_C, "xiangqi_skeleton.c"),

        ("poker", ProgrammingLanguage::Rust) => (POKER_RS, "poker_skeleton.rs"),
        ("poker", ProgrammingLanguage::Go) => (POKER_GO, "poker_skeleton.go"),
        ("poker", ProgrammingLanguage::C) => (POKER_C, "poker_skeleton.c"),

        _ => {
            return Err(ApiError::NotFound(format!(
                "No starter code available for {} in {}",
                game.id,
                lang.to_extension(),
            )));
        }
    };

    Ok(Json(StarterCodeResponse {
        game_id: game.id.to_string(),
        language: language.to_lowercase(),
        filename: filename.to_string(),
        code: code.to_string(),
    }))
}
