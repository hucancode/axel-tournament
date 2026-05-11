/// Per-game configuration helpers. Each game owns its own keys; the api
/// stores the blob verbatim and never validates. Backend logic falls back
/// to defaults when a value is missing or out of range, so unknown keys
/// are safe to send.

export interface GameConfigField {
  key: string;
  label: string;
  type: 'number' | 'boolean';
  min?: number;
  max?: number;
  step?: number;
  default: number | boolean;
  help?: string;
}

const TTT_FIELDS: GameConfigField[] = [
  {
    key: 'board_size',
    label: 'Board size (NxN)',
    type: 'number',
    min: 3,
    max: 32,
    step: 1,
    default: 16,
    help: 'Side length of the square board (3–32).',
  },
  {
    key: 'win_chain',
    label: 'In a row to win',
    type: 'number',
    min: 3,
    max: 32,
    step: 1,
    default: 5,
    help: 'Number of marks in a row needed to win (3 to board size).',
  },
];

const ROUNDS_FIELDS = (def: number, max = 1000): GameConfigField[] => [
  {
    key: 'rounds',
    label: 'Rounds per match',
    type: 'number',
    min: 1,
    max,
    step: 1,
    default: def,
  },
];

const BOARD_TIME_FIELDS: GameConfigField[] = [
  {
    key: 'time_pool_minutes',
    label: 'Time pool (minutes per player)',
    type: 'number',
    min: 0,
    max: 600,
    step: 1,
    default: 0,
    help: 'Total clock per player. 0 = no pool limit.',
  },
  {
    key: 'time_per_turn_seconds',
    label: 'Per-turn limit (seconds)',
    type: 'number',
    min: 0,
    max: 3600,
    step: 1,
    default: 0,
    help: 'Hard timeout per move. 0 = no per-turn limit.',
  },
  {
    key: 'blind',
    label: 'Blind variant',
    type: 'boolean',
    default: false,
    help: 'Shuffle non-king pieces with hidden identity. They move as the piece originally on their square until first move reveals them.',
  },
];

const SCHEMAS: Record<string, GameConfigField[]> = {
  'tic-tac-toe': TTT_FIELDS,
  'rock-paper-scissors': ROUNDS_FIELDS(5),
  'prisoners-dilemma': ROUNDS_FIELDS(10),
  chess: BOARD_TIME_FIELDS,
  xiangqi: BOARD_TIME_FIELDS,
};

export function configFieldsFor(gameId: string): GameConfigField[] {
  return SCHEMAS[gameId] ?? [];
}

/// Build the START payload string the judge logic expects for a game.
/// Returns '' when the game has no configurable parameters; `roomSocket.act`
/// then sends a bare `ACT START`.
export function startPayload(
  gameId: string,
  config: Record<string, unknown> | undefined,
): string {
  const cfg = config ?? {};
  switch (gameId) {
    case 'tic-tac-toe': {
      const size = numOr(cfg.board_size, 16);
      const chain = numOr(cfg.win_chain, 5);
      return `${size} ${chain}`;
    }
    case 'rock-paper-scissors':
    case 'prisoners-dilemma': {
      const rounds = numOr(cfg.rounds, gameId === 'rock-paper-scissors' ? 5 : 10);
      return `${rounds}`;
    }
    case 'chess':
    case 'xiangqi': {
      // 0 is the explicit "no limit" value here, so we accept it.
      const pool = nonNegNumOr(cfg.time_pool_minutes, 0);
      const perTurn = nonNegNumOr(cfg.time_per_turn_seconds, 0);
      const blind = cfg.blind ? 1 : 0;
      return `${pool} ${perTurn} ${blind}`;
    }
    default:
      return '';
  }
}

function numOr(v: unknown, def: number): number {
  const n = typeof v === 'number' ? v : Number(v);
  return Number.isFinite(n) && n > 0 ? n : def;
}

function nonNegNumOr(v: unknown, def: number): number {
  const n = typeof v === 'number' ? v : Number(v);
  return Number.isFinite(n) && n >= 0 ? n : def;
}

export function defaultConfig(gameId: string): Record<string, unknown> {
  const out: Record<string, unknown> = {};
  for (const f of configFieldsFor(gameId)) {
    out[f.key] = f.default;
  }
  return out;
}
