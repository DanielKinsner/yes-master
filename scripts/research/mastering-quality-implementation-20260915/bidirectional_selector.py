"""Separately versioned bounded C1 selector; the historical v1 is untouched."""
from preserving_selector import Candidate, ResolvedSelection, TARGET_TOLERANCE_LU

VERSION = 'control-inclusive-bidirectional-v1'


def select(candidates, context):
    rows = tuple(candidates)
    assert rows and all(r.context == context for r in rows), 'mixed or missing context'
    assert len({r.id for r in rows}) == len(rows), 'duplicate IDs'
    assert len({r.target_lufs for r in rows}) == 1, 'mixed target'
    assert len({r.ceiling for r in rows}) == 1, 'mixed ceiling'
    identities = {(r.policy, r.offset_db) for r in rows}
    initial = {('control', 0.), ('single', 0.)}
    complete = initial | {('single', -3.), ('single', 3.)}
    assert identities in (initial, complete) and len(identities) == len(rows), 'incomplete or invalid grid'
    assert all(isinstance(r.character_failures, tuple) and isinstance(r.technical_failures, tuple)
               for r in rows), 'explicit immutable measurement status required'
    valid = [r for r in rows if not r.errors()]
    rejected = tuple(sorted((r.id, r.errors()) for r in rows if r.errors()))
    qualified = [r for r in valid if not r.character_failures]
    hits = [r for r in qualified if abs(r.lufs-r.target_lufs) <= TARGET_TOLERANCE_LU]
    tie = lambda r: (r.policy != 'control', abs(r.offset_db), r.offset_db, r.id)
    control = next(r for r in rows if r.policy == 'control')
    if hits:
        winner, reason = min(hits, key=tie), 'qualified_target'
    elif qualified:
        winner = min(qualified, key=lambda r: (abs(r.lufs-r.target_lufs), *tie(r)))
        reason = 'qualified_character_target_shortfall'
    elif control in valid:
        winner, reason = control, 'character_fallback'
    else:
        winner, reason = None, 'no_verified_fallback'
    character_ok = winner is not None and not winner.character_failures
    target_ok = winner is not None and abs(winner.lufs-winner.target_lufs) <= TARGET_TOLERANCE_LU
    return ResolvedSelection(context, VERSION, winner, reason, character_ok, target_ok,
                             identities == initial and not (character_ok and target_ok), rejected)
