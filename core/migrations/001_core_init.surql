DEFINE TABLE IF NOT EXISTS user SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS email ON user TYPE string ASSERT string::is_email($value);
DEFINE FIELD IF NOT EXISTS username ON user TYPE string;
DEFINE FIELD IF NOT EXISTS password_hash ON user TYPE option<string>;
DEFINE FIELD IF NOT EXISTS role ON user TYPE string;
DEFINE FIELD IF NOT EXISTS location ON user TYPE string;
DEFINE FIELD IF NOT EXISTS oauth_provider ON user TYPE option<string>;
DEFINE FIELD IF NOT EXISTS oauth_id ON user TYPE option<string>;
DEFINE FIELD IF NOT EXISTS is_banned ON user TYPE bool DEFAULT false;
DEFINE FIELD IF NOT EXISTS ban_reason ON user TYPE option<string>;
DEFINE FIELD IF NOT EXISTS created_at ON user TYPE datetime;
DEFINE FIELD IF NOT EXISTS updated_at ON user TYPE datetime;
DEFINE FIELD IF NOT EXISTS password_reset_token ON user TYPE option<string>;
DEFINE FIELD IF NOT EXISTS password_reset_expires ON user TYPE option<datetime>;
DEFINE INDEX IF NOT EXISTS unique_email ON user COLUMNS email UNIQUE;
DEFINE INDEX IF NOT EXISTS unique_username ON user COLUMNS username UNIQUE;

DEFINE TABLE IF NOT EXISTS tournament SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS game_id ON tournament TYPE string;
DEFINE FIELD IF NOT EXISTS name ON tournament TYPE string;
DEFINE FIELD IF NOT EXISTS description ON tournament TYPE string;
DEFINE FIELD IF NOT EXISTS status ON tournament TYPE string;
DEFINE FIELD IF NOT EXISTS min_players ON tournament TYPE number;
DEFINE FIELD IF NOT EXISTS max_players ON tournament TYPE number;
DEFINE FIELD IF NOT EXISTS start_time ON tournament TYPE option<datetime>;
DEFINE FIELD IF NOT EXISTS end_time ON tournament TYPE option<datetime>;
DEFINE FIELD IF NOT EXISTS match_generation_type ON tournament TYPE string DEFAULT 'all_vs_all';
DEFINE FIELD IF NOT EXISTS kind ON tournament TYPE string DEFAULT 'bot';
DEFINE FIELD IF NOT EXISTS config ON tournament TYPE object FLEXIBLE DEFAULT {};
DEFINE FIELD IF NOT EXISTS created_at ON tournament TYPE datetime;
DEFINE FIELD IF NOT EXISTS updated_at ON tournament TYPE datetime;

DEFINE TABLE IF NOT EXISTS tournament_participant SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS tournament_id ON tournament_participant TYPE record<tournament>;
DEFINE FIELD IF NOT EXISTS user_id ON tournament_participant TYPE record<user>;
DEFINE FIELD IF NOT EXISTS submission_id ON tournament_participant TYPE option<record<submission>>;
DEFINE FIELD IF NOT EXISTS score ON tournament_participant TYPE number DEFAULT 0;
DEFINE FIELD IF NOT EXISTS wins ON tournament_participant TYPE number DEFAULT 0;
DEFINE FIELD IF NOT EXISTS losses ON tournament_participant TYPE number DEFAULT 0;
DEFINE FIELD IF NOT EXISTS draws ON tournament_participant TYPE number DEFAULT 0;
DEFINE FIELD IF NOT EXISTS elo ON tournament_participant TYPE option<number>;
DEFINE FIELD IF NOT EXISTS rank ON tournament_participant TYPE option<number>;
DEFINE FIELD IF NOT EXISTS joined_at ON tournament_participant TYPE datetime;
DEFINE INDEX IF NOT EXISTS unique_tournament_user ON tournament_participant COLUMNS tournament_id, user_id UNIQUE;

DEFINE TABLE IF NOT EXISTS submission SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS user_id ON submission TYPE record<user>;
DEFINE FIELD IF NOT EXISTS tournament_id ON submission TYPE record<tournament>;
DEFINE FIELD IF NOT EXISTS game_id ON submission TYPE string;
DEFINE FIELD IF NOT EXISTS language ON submission TYPE string;
DEFINE FIELD IF NOT EXISTS code ON submission TYPE string;
DEFINE FIELD IF NOT EXISTS status ON submission TYPE string DEFAULT 'pending';
DEFINE FIELD IF NOT EXISTS error_message ON submission TYPE option<string>;
DEFINE FIELD IF NOT EXISTS compiled_binary_path ON submission TYPE option<string>;
DEFINE FIELD IF NOT EXISTS created_at ON submission TYPE datetime;

DEFINE TABLE IF NOT EXISTS room SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS game_id ON room TYPE string;
DEFINE FIELD IF NOT EXISTS host_id ON room TYPE record<user>;
DEFINE FIELD IF NOT EXISTS name ON room TYPE string;
DEFINE FIELD IF NOT EXISTS max_players ON room TYPE number;
DEFINE FIELD IF NOT EXISTS status ON room TYPE string;
DEFINE FIELD IF NOT EXISTS players ON room TYPE array<record<user>>;
DEFINE FIELD IF NOT EXISTS human_timeout_ms ON room TYPE option<number>;
DEFINE FIELD IF NOT EXISTS tournament_id ON room TYPE option<record<tournament>>;
DEFINE FIELD IF NOT EXISTS allowed_user_ids ON room TYPE array<record<user>> DEFAULT [];
DEFINE FIELD IF NOT EXISTS is_ranked ON room TYPE bool DEFAULT false;
DEFINE FIELD IF NOT EXISTS winner_id ON room TYPE option<record<user>>;
DEFINE FIELD IF NOT EXISTS created_at ON room TYPE datetime;
DEFINE FIELD IF NOT EXISTS updated_at ON room TYPE datetime;
DEFINE FIELD IF NOT EXISTS event_history ON room TYPE array<string> DEFAULT [];
DEFINE FIELD IF NOT EXISTS config ON room TYPE object FLEXIBLE DEFAULT {};
DEFINE INDEX IF NOT EXISTS idx_room_game ON room COLUMNS game_id;
DEFINE INDEX IF NOT EXISTS idx_room_status ON room COLUMNS status;
DEFINE INDEX IF NOT EXISTS idx_room_tournament ON room COLUMNS tournament_id;

DEFINE TABLE IF NOT EXISTS matchmaking_ticket SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS tournament_id ON matchmaking_ticket TYPE record<tournament>;
DEFINE FIELD IF NOT EXISTS user_id ON matchmaking_ticket TYPE record<user>;
DEFINE FIELD IF NOT EXISTS elo ON matchmaking_ticket TYPE number;
DEFINE FIELD IF NOT EXISTS queued_at ON matchmaking_ticket TYPE datetime;
DEFINE INDEX IF NOT EXISTS idx_mm_ticket_unique
    ON matchmaking_ticket COLUMNS tournament_id, user_id UNIQUE;

DEFINE TABLE IF NOT EXISTS match SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS tournament_id ON match TYPE option<record<tournament>>;
DEFINE FIELD IF NOT EXISTS game_id ON match TYPE string;
DEFINE FIELD IF NOT EXISTS room_id ON match TYPE option<record<room>>;
DEFINE FIELD IF NOT EXISTS status ON match TYPE string;
DEFINE FIELD OVERWRITE participants ON match TYPE array<{
    user_id: record<user>,
    submission_id: option<record<submission>>,
    score: option<float>
}>;
DEFINE FIELD IF NOT EXISTS metadata ON match TYPE option<object>;
DEFINE FIELD IF NOT EXISTS error_message ON match TYPE option<string> DEFAULT NONE;
DEFINE FIELD IF NOT EXISTS faulted_user_ids ON match TYPE array<record<user>> DEFAULT [];
DEFINE FIELD IF NOT EXISTS round ON match TYPE option<number>;
DEFINE FIELD IF NOT EXISTS bracket ON match TYPE option<string>;
DEFINE FIELD IF NOT EXISTS bracket_position ON match TYPE option<number>;
DEFINE FIELD IF NOT EXISTS elo_applied ON match TYPE bool DEFAULT false;
DEFINE FIELD IF NOT EXISTS game_event_source ON match TYPE option<string>;
DEFINE FIELD IF NOT EXISTS judge_server_name ON match TYPE option<string>;
DEFINE FIELD IF NOT EXISTS created_at ON match TYPE datetime;
DEFINE FIELD IF NOT EXISTS updated_at ON match TYPE datetime;
DEFINE FIELD IF NOT EXISTS started_at ON match TYPE option<datetime>;
DEFINE FIELD IF NOT EXISTS completed_at ON match TYPE option<datetime>;
DEFINE INDEX IF NOT EXISTS idx_match_tournament ON match COLUMNS tournament_id;
DEFINE INDEX IF NOT EXISTS idx_match_room ON match COLUMNS room_id;
DEFINE INDEX IF NOT EXISTS idx_match_status ON match COLUMNS status;
DEFINE INDEX IF NOT EXISTS idx_match_created ON match COLUMNS created_at;

DEFINE INDEX IF NOT EXISTS idx_tournament_status ON tournament COLUMNS status;
DEFINE INDEX IF NOT EXISTS idx_tournament_game ON tournament COLUMNS game_id;
DEFINE INDEX IF NOT EXISTS idx_tournament_created ON tournament COLUMNS created_at;
DEFINE INDEX IF NOT EXISTS idx_submission_user ON submission COLUMNS user_id;
DEFINE INDEX IF NOT EXISTS idx_submission_tournament ON submission COLUMNS tournament_id;
