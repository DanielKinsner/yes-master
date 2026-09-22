"""Grid, safety, deterministic selection and stopping-policy regressions."""
from dataclasses import replace
from itertools import permutations
import json
from pathlib import Path
import unittest

from bidirectional_selector import Candidate, select


class BidirectionalTests(unittest.TestCase):
    def setUp(self):
        c = Candidate('source/settings/version', 'control', 'a'*64, 'control', 0.,
                      -14., -14., -1., -1., ('attack',))
        self.rows = [c, replace(c, id='zero', policy='single', character_failures=('tone',)),
                     replace(c, id='negative', policy='single', offset_db=-3., character_failures=('tone',)),
                     replace(c, id='positive', policy='single', offset_db=3., character_failures=())]

    def resolve(self, rows):
        return select(rows, self.rows[0].context)

    def test_positive_resolves_without_rewriting_old_grid(self):
        self.assertEqual(self.resolve(self.rows).selected.id, 'positive')
        from preserving_selector import select as historical
        with self.assertRaises(AssertionError):
            historical(self.rows, self.rows[0].context)

    def test_order_independence(self):
        expected = self.resolve(self.rows)
        for rows in permutations(self.rows):
            self.assertEqual(self.resolve(rows), expected)

    def test_qualified_control_wins(self):
        self.rows[0] = replace(self.rows[0], character_failures=())
        self.assertEqual(self.resolve(self.rows).selected.id, 'control')
        self.assertFalse(self.resolve(self.rows[:2]).needs_more_search)

    def test_missing_one_direction_rejected(self):
        with self.assertRaises(AssertionError):
            self.resolve(self.rows[:3])

    def test_shortfall_prefers_qualified_current(self):
        self.rows[0] = replace(self.rows[0], character_failures=(), lufs=-18.)
        self.rows[3] = replace(self.rows[3], lufs=-19.)
        result = self.resolve(self.rows)
        self.assertEqual(result.selected.id, 'control')
        self.assertEqual(result.reason, 'qualified_character_target_shortfall')
        self.assertFalse(result.needs_more_search)

    def test_invalid_winner_and_control_never_fabricate_fallback(self):
        self.rows[3] = replace(self.rows[3], peak=-.9)
        self.rows[0] = replace(self.rows[0], technical_failures=('unverified',))
        result = self.resolve(self.rows)
        self.assertIsNone(result.selected)
        self.assertEqual(result.reason, 'no_verified_fallback')

    def test_nonfinite_and_missing_character_rejected(self):
        for changes in ({'lufs': float('nan')}, {'character_failures': ('attack:unavailable',)}):
            rows = self.rows[:3] + [replace(self.rows[3], **changes)]
            result = self.resolve(rows)
            self.assertEqual(result.reason, 'character_fallback')
            self.assertFalse(result.character_qualified)

    def test_context_target_ceiling_and_duplicate_rejected(self):
        for changes in ({'context': 'other'}, {'target_lufs': -9.}, {'ceiling': -.5},
                        {'id': 'control'}, {'offset_db': -6.}):
            with self.assertRaises(AssertionError):
                self.resolve(self.rows[:3]+[replace(self.rows[3], **changes)])

    def test_bounded_pair_and_negative_tie(self):
        self.assertTrue(self.resolve(self.rows[:2]).needs_more_search)
        self.rows[2] = replace(self.rows[2], character_failures=())
        self.assertEqual(self.resolve(self.rows).selected.id, 'negative')
        self.rows[2] = replace(self.rows[2], character_failures=('tone',))
        self.rows[3] = replace(self.rows[3], character_failures=('tone',))
        self.assertFalse(self.resolve(self.rows).needs_more_search)

    def test_verified_development_results_preserve_shortfalls_and_controls(self):
        path = Path(__file__).resolve().parents[3]/'docs/reviews/evidence/2026-09-21-dynamics-research/bidirectional-v1.json'
        data = json.loads(path.read_text(encoding='utf-8'))
        expected = {'piano': ('single', -3., False), 'imaginal': ('control', 0., False),
                    'metal': ('single', 3., True), 'rich': ('control', 0., False)}
        for name, result in expected.items():
            rows = []
            for item in data['rows']:
                row = item['native']
                if row['case'] == name:
                    rows.append(Candidate(name, row['id'], row['sha256'], row['policy'], row['offset_db'],
                        row['lufs'], -14., row['peak'], row['ceiling'], tuple(item['metric']['character_failures'])))
            selected = select(rows, name)
            self.assertTrue(selected.character_qualified)
            self.assertEqual((selected.selected.policy, selected.selected.offset_db, selected.target_feasible), result)
            self.assertFalse(selected.needs_more_search)

    def quiet_rows(self, name, gain, reference='character_failures'):
        path = Path(__file__).resolve().parents[3]/'docs/reviews/evidence/2026-09-21-dynamics-research/quiet-copy-v2.json'
        data = json.loads(path.read_text(encoding='utf-8'))
        rows = []
        for item in data['rows']:
            row = item['native']
            if (row['case'], row['source_gain_db']) != (name, gain):
                continue
            self.assertTrue(item['independent']['peak_pass'])
            if row['policy'] == 'single':
                self.assertTrue(item['metric']['numeric_consistency']['pass_all'])
            available = row['lufs'] is not None
            if available:
                self.assertTrue(item['independent']['lufs_pass'])
            technical = [] if available else ['unavailable_loudness_for_selector']
            if not item['independent']['lufs_pass']:
                technical.append('independent_loudness_failed')
            rows.append(Candidate(name, row['id'], row['sha256'], row['policy'], row['offset_db'],
                row['lufs'] if available else -float('inf'), -14., row['peak'], row['ceiling'],
                tuple(item['metric'][reference]), tuple(technical)))
        self.assertEqual(len(rows), 4)
        return rows

    def test_below_gate_controls_do_not_fabricate_selector_fallback(self):
        for name in ('piano', 'imaginal', 'metal', 'rich'):
            rows = self.quiet_rows(name, -80.)
            result = select(rows, name)
            control = next(r for r in rows if r.policy == 'control')
            self.assertIn('unavailable_loudness_for_selector', dict(result.rejected)[control.id])
            if name in ('imaginal', 'rich'):
                self.assertIsNone(result.selected)
                self.assertEqual(result.reason, 'no_verified_fallback')
            else:
                self.assertEqual(result.selected.policy, 'single')
                self.assertTrue(result.character_qualified)

    def test_verified_metal_reference_shift_changes_choice_despite_consistent_pcm(self):
        for gain in (-40., -80.):
            matched = select(self.quiet_rows('metal', gain), 'metal')
            fixed = select(self.quiet_rows('metal', gain, 'fixed_original_reference_failures'), 'metal')
            self.assertEqual(matched.selected.offset_db, 0.)
            self.assertEqual(fixed.selected.offset_db, 3.)
            self.assertTrue(matched.target_feasible and fixed.target_feasible)


if __name__ == '__main__':
    unittest.main()
