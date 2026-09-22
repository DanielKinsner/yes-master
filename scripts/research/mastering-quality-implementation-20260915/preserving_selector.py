"""Pure research selection over whole-file-verified candidates; no DSP or UI calls."""
from dataclasses import dataclass
import math

VERSION = 'control-inclusive-preserving-v1'
TARGET_TOLERANCE_LU = .2


@dataclass(frozen=True)
class Candidate:
    context: str
    id: str
    sha256: str
    policy: str
    offset_db: float
    lufs: float
    target_lufs: float
    peak: float
    ceiling: float
    character_failures: tuple[str, ...]
    technical_failures: tuple[str, ...] = ()

    def errors(self):
        errors = list(self.technical_failures)
        if not all(math.isfinite(value) for value in
                   (self.offset_db,self.lufs,self.target_lufs,self.peak,self.ceiling)):
            errors.append('nonfinite_delivery')
        elif self.peak > self.ceiling:
            errors.append('over_ceiling')
        return tuple(errors)


@dataclass(frozen=True)
class ResolvedSelection:
    context: str
    algorithm: str
    selected: Candidate | None
    reason: str
    character_qualified: bool
    target_feasible: bool
    needs_more_search: bool
    rejected: tuple[tuple[str, tuple[str, ...]], ...]


def select(candidates, context):
    candidates = tuple(candidates)
    assert candidates and all(row.context == context for row in candidates), 'mixed or missing context'
    assert len({row.id for row in candidates}) == len(candidates), 'duplicate IDs'
    assert len({row.target_lufs for row in candidates}) == 1, 'mixed target'
    controls = [row for row in candidates if row.policy == 'control']
    assert len(controls) == 1 and controls[0].offset_db == 0, 'one exact control required'
    assert all(row.policy in ('control','single') and row.offset_db in (0.,-3.,-6.,-12.)
               for row in candidates), 'outside frozen candidate grid'
    assert all(isinstance(row.character_failures,tuple) and isinstance(row.technical_failures,tuple)
               for row in candidates), 'explicit immutable measurement status required'
    valid = [row for row in candidates if not row.errors()]
    rejected = tuple(sorted((row.id,row.errors()) for row in candidates if row.errors()))
    qualified = [row for row in valid if not row.character_failures]
    hits = [row for row in qualified if abs(row.lufs-row.target_lufs) <= TARGET_TOLERANCE_LU]
    tie = lambda row: (row.policy != 'control', abs(row.offset_db), row.id)
    if hits:
        winner, reason = min(hits,key=tie), 'qualified_target'
    elif qualified:
        winner = min(qualified,key=lambda row: (abs(row.lufs-row.target_lufs),*tie(row)))
        reason = 'qualified_character_target_shortfall'
    elif controls[0] in valid:
        winner, reason = controls[0], 'character_fallback'
    else:
        winner, reason = None, 'no_verified_fallback'
    character_ok = winner is not None and not winner.character_failures
    target_ok = winner is not None and abs(winner.lufs-winner.target_lufs) <= TARGET_TOLERANCE_LU
    return ResolvedSelection(context,VERSION,winner,reason,character_ok,target_ok,
                             not (character_ok and target_ok),rejected)
