"""Grid, safety, deterministic selection and stopping-policy regressions."""
from dataclasses import replace
from itertools import permutations
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


if __name__ == '__main__':
    unittest.main()
