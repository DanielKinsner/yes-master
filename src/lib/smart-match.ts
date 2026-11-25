import type {
  MasteringSettings,
  SmartMatchBandReadout,
  SmartMatchReadout,
  SmartMatchReference,
  SmartMatchRefusal,
  SmartMatchSettings,
} from "../bindings";
import { SMART_MATCH_DEFAULT_AMOUNT, SMART_MATCH_REFUSAL_REASONS } from "../bindings";

// Smart Match's interface logic (Smart Match plan, Slice 7). Every number
// shown comes from the backend readout (plan M17); this file only words it,
// using the plan's locked copy exactly.

/// Names longer than this are shortened with an ellipsis (locked copy); the
/// full name stays available on hover and focus.
export const SMART_MATCH_NAME_MAX = 60;
/// Moves this size or larger are named, largest first, at most three
/// (locked copy). The backend's dead zone is the same 0.3 dB, so every move
/// the chain makes is one this can name.
export const NAMED_MOVE_MIN_DB = 0.3;
export const NAMED_MOVES_MAX = 3;

export const SMART_MATCH_BAND_LABELS = [
  "Sub",
  "Bass",
  "Low mids",
  "Mids",
  "Presence",
  "Air",
] as const;

/// The locked sentence phrases, per band: [cut, boost], with X for the amount.
const SENTENCE_PHRASES: readonly (readonly [string, string])[] = [
  ["tamed X dB of sub rumble", "added X dB of deep bass"],
  ["tamed X dB of boom", "added X dB of bass"],
  ["cleared X dB of mud", "added X dB of warmth"],
  ["eased X dB of midrange", "added X dB of midrange"],
  ["softened X dB of harshness", "added X dB of presence"],
  ["softened X dB of brightness", "added X dB of air"],
];

export const SMART_MATCH_COPY = {
  label: "Smart Match",
  tag: "Beta",
  choose: "Choose reference…",
  change: "Change…",
  remove: "Remove reference",
  noReference: "Pick a song you want this track to sit next to.",
  measuring: "Listening to your reference…",
  trackNotReady: "Listening to your track…",
  noProfile: "Not enough audio to analyze this track.",
  zeroAmount: "Match amount is at 0, so Smart Match is idle.",
  album: "Smart Match applies to Track Master, not Album renders.",
  partial: " Your reference is very different from this track, so the match is partial.",
  amountLabel: "Match amount",
  stripCaption: "Tone only. Loudness and width are not matched.",
  stripCaptionDynamics:
    "Moves are applied before the compressor and limiter, so the final change can be smaller on loud material.",
} as const;

/// The reference's name as shown: shortened with an ellipsis beyond 60
/// characters.
export function displayReferenceName(name: string): string {
  const chars = Array.from(name);
  return chars.length > SMART_MATCH_NAME_MAX
    ? `${chars.slice(0, SMART_MATCH_NAME_MAX - 1).join("")}…`
    : name;
}

/// The bands to name: 0.3 dB or more, largest first, at most three; ties keep
/// band order. Mirrors `smart_match::named_moves` in Rust (the receipt).
export function namedMoves(corrections: readonly number[]): { band: number; value: number }[] {
  return corrections
    .map((value, band) => ({ band, value }))
    .filter((m) => Math.abs(m.value) >= NAMED_MOVE_MIN_DB)
    .sort((a, b) => Math.abs(b.value) - Math.abs(a.value) || a.band - b.band)
    .slice(0, NAMED_MOVES_MAX);
}

/// `Matching "<name>": cleared 1.2 dB of mud, added 0.8 dB of air.`
export function correctionSentence(name: string, corrections: readonly number[]): string {
  const moves = namedMoves(corrections).map(({ band, value }) => {
    const [cut, boost] = SENTENCE_PHRASES[band];
    return (value < 0 ? cut : boost).replace("X", Math.abs(value).toFixed(1));
  });
  return `Matching "${displayReferenceName(name)}": ${moves.join(", ")}.`;
}

/// The full refusal line for a reason.
export function refusalLine(reason: SmartMatchRefusal): string {
  return `That file can't be used as a reference: ${SMART_MATCH_REFUSAL_REASONS[reason]}.`;
}

export type SmartMatchUiState =
  | "no_reference"
  | "measuring"
  | "track_not_ready"
  | "no_profile"
  | "applied"
  | "nothing_to_change"
  | "switched_off"
  | "zero_amount"
  | "album_disabled";

/// Everything the Smart Match row and the Advanced strip show.
export interface SmartMatchView {
  state: SmartMatchUiState;
  /// The reference's full name, or null without a reference.
  name: string | null;
  enabled: boolean;
  amount: number;
  /// The result line (locked copy). A refusal replaces it until the next pick.
  line: string;
  /// True when the line includes the partial notice.
  partial: boolean;
  /// True while a pick is being measured (controls busy).
  measuring: boolean;
  /// True when the numbers are from the previous settings and a fresh
  /// readout is on its way (presentation only).
  pending: boolean;
  /// Six bands from the readout, or null when there are no numbers to show.
  bands: SmartMatchBandReadout[] | null;
}

export interface SmartMatchViewInput {
  settings: MasteringSettings;
  /// The readout's Smart Match section for these settings, or the last one
  /// for this track while a fresh one is pending (`pending` true).
  readout: SmartMatchReadout | null | undefined;
  pending: boolean;
  album: boolean;
  measuring: boolean;
  refusal: SmartMatchRefusal | null;
  /// The track's analysis has finished.
  analysisReady: boolean;
  /// That analysis produced a six-band profile.
  analysisHasProfile: boolean;
}

export function smartMatchView(input: SmartMatchViewInput): SmartMatchView {
  const state = input.settings.advanced.smart_match ?? null;
  const reference = state?.reference ?? null;
  const name = reference?.name ?? null;
  const enabled = state?.enabled ?? true;
  const amount = state?.amount ?? SMART_MATCH_DEFAULT_AMOUNT;
  const base = {
    name,
    enabled,
    amount,
    measuring: input.measuring,
    pending: false,
    partial: false,
    bands: input.readout?.bands ?? null,
  };
  const withRefusal = (view: SmartMatchView): SmartMatchView =>
    input.refusal ? { ...view, line: refusalLine(input.refusal), partial: false } : view;

  if (input.album) {
    return { ...base, bands: null, state: "album_disabled", line: SMART_MATCH_COPY.album };
  }
  if (input.measuring) {
    return { ...base, state: "measuring", line: SMART_MATCH_COPY.measuring };
  }
  if (!reference || name === null) {
    return withRefusal({ ...base, bands: null, state: "no_reference", line: SMART_MATCH_COPY.noReference });
  }
  const shown = displayReferenceName(name);
  if (!enabled) {
    return withRefusal({ ...base, state: "switched_off", line: `Match to "${shown}" is off.` });
  }
  if (amount <= 0) {
    return withRefusal({ ...base, state: "zero_amount", line: SMART_MATCH_COPY.zeroAmount });
  }
  const readout = input.readout;
  const readoutState = readout?.state;
  if (!readout || readoutState === "track_not_ready" || readoutState === "no_reference") {
    const noProfile = input.analysisReady && !input.analysisHasProfile;
    return withRefusal({
      ...base,
      bands: readout?.bands ?? null,
      state: noProfile ? "no_profile" : "track_not_ready",
      line: noProfile ? SMART_MATCH_COPY.noProfile : SMART_MATCH_COPY.trackNotReady,
    });
  }
  const corrections = readout.bands.map((b) => b.correction_db);
  if (readoutState === "applied" && namedMoves(corrections).length > 0) {
    const sentence = correctionSentence(name, corrections);
    return withRefusal({
      ...base,
      pending: input.pending,
      state: "applied",
      partial: readout.partial,
      line: readout.partial ? sentence + SMART_MATCH_COPY.partial : sentence,
    });
  }
  return withRefusal({
    ...base,
    pending: input.pending,
    state: "nothing_to_change",
    line: `Already close to "${shown}". Nothing to change.`,
  });
}

// ---------------------------------------------------------------------------
// Settings edits. Each is one settings mutation, so one undo step.
// ---------------------------------------------------------------------------

function withState(s: MasteringSettings, next: SmartMatchSettings | undefined): MasteringSettings {
  const advanced = { ...s.advanced };
  if (next) {
    advanced.smart_match = next;
  } else {
    delete advanced.smart_match;
  }
  return { ...s, advanced };
}

/// A newly chosen reference starts switched on. The amount, an Advanced
/// value, is kept.
export function withSmartMatchReference(
  s: MasteringSettings,
  reference: SmartMatchReference,
): MasteringSettings {
  const amount = s.advanced.smart_match?.amount ?? SMART_MATCH_DEFAULT_AMOUNT;
  return withState(s, { reference, enabled: true, amount });
}

export function withSmartMatchEnabled(s: MasteringSettings, enabled: boolean): MasteringSettings {
  const state = s.advanced.smart_match;
  if (!state) return s;
  return withState(s, { ...state, enabled });
}

/// Removing the reference keeps a non-default amount (an Advanced edit);
/// otherwise no Smart Match state is left behind.
export function withoutSmartMatchReference(s: MasteringSettings): MasteringSettings {
  const amount = s.advanced.smart_match?.amount ?? SMART_MATCH_DEFAULT_AMOUNT;
  return withState(
    s,
    amount === SMART_MATCH_DEFAULT_AMOUNT ? undefined : { reference: null, enabled: true, amount },
  );
}

export function withSmartMatchAmount(s: MasteringSettings, amount: number): MasteringSettings {
  const clamped = Math.min(1, Math.max(0, Number.isFinite(amount) ? amount : SMART_MATCH_DEFAULT_AMOUNT));
  const state = s.advanced.smart_match;
  return withState(s, {
    reference: state?.reference ?? null,
    enabled: state?.enabled ?? true,
    amount: clamped,
  });
}

/// Standard-managed reset (plan M13): the reference and its switch stay; a
/// non-default amount goes back to 50%. Leaves no empty state behind.
export function resetSmartMatchAmount(s: MasteringSettings): MasteringSettings {
  const state = s.advanced.smart_match;
  if (!state) return s;
  if (!state.reference) return withState(s, undefined);
  return withState(s, { ...state, amount: SMART_MATCH_DEFAULT_AMOUNT });
}

/// True when the amount is not the default: an Advanced (non-managed) edit.
export function hasSmartMatchAmountEdit(s: MasteringSettings): boolean {
  return (s.advanced.smart_match?.amount ?? SMART_MATCH_DEFAULT_AMOUNT) !== SMART_MATCH_DEFAULT_AMOUNT;
}

/**
 * A saved preset's settings applied to a song. A Smart Match reference belongs
 * to the song, never to a preset (Smart Match plan, M12): whatever the preset
 * carries is dropped (the backend already strips it on save and on read), and
 * the song's current Smart Match state is kept, absent included.
 */
export function presetSettingsKeepingSmartMatch(
  preset: MasteringSettings,
  current: SmartMatchSettings | null | undefined,
): MasteringSettings {
  const advanced = { ...preset.advanced };
  delete advanced.smart_match;
  if (current) {
    advanced.smart_match = current;
  }
  return { ...preset, advanced };
}
