"""Selection regressions over verified development facts and invalid witnesses."""
from dataclasses import FrozenInstanceError, replace
import itertools
from pathlib import Path
import unittest

from evaluate_preserving_selector import load_candidates
from preserving_selector import select


class PreservingSelectorTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        root = Path(__file__).resolve().parents[3]
        cls.cases = load_candidates(root/'docs/reviews/evidence/2026-09-16-dynamics-research/verification.json')

    def resolve(self, rows):
        return select(rows, rows[0].context)

    def test_rich_keeps_qualified_control_instead_of_quieter_new_candidate(self):
        selected = self.resolve(self.cases['rich'])
        self.assertEqual(selected.selected.id, 'rich-control')
        self.assertTrue(selected.character_qualified)
        self.assertFalse(selected.target_feasible)
        self.assertEqual(selected.reason, 'qualified_character_target_shortfall')

    def test_metal_fallback_does_not_masquerade_as_character_pass(self):
        selected = self.resolve(self.cases['metal'])
        self.assertEqual(selected.selected.id, 'metal-control')
        self.assertEqual(selected.reason, 'character_fallback')
        self.assertFalse(selected.character_qualified)
        self.assertTrue(selected.target_feasible)
        self.assertTrue(selected.needs_more_search)

    def test_three_qualified_single_candidates_remain_selected(self):
        for name in ('funk','aphelion','baby'):
            selected = self.resolve(self.cases[name])
            self.assertEqual(selected.selected.id, name+'-single')
            self.assertTrue(selected.character_qualified and selected.target_feasible)
            self.assertFalse(selected.needs_more_search)

    def test_iteration_order_cannot_change_selection(self):
        for rows in self.cases.values():
            expected = self.resolve(rows)
            for order in itertools.permutations(rows):
                self.assertEqual(self.resolve(order), expected)

    def test_verified_current_processing_wins_qualified_target_tie(self):
        rows = self.cases['baby']
        rows = [replace(row,lufs=-14.) if row.policy == 'control' else row for row in rows]
        self.assertEqual(self.resolve(rows).selected.id,'baby-control')

    def test_over_ceiling_target_hit_cannot_win(self):
        rows = [replace(row,peak=-.9) if row.policy == 'single' else row for row in self.cases['baby']]
        selected = self.resolve(rows)
        self.assertEqual(selected.selected.id,'baby-control')
        self.assertIn(('baby-single',('over_ceiling',)),selected.rejected)

    def test_unavailable_character_measurement_is_a_failure(self):
        rows = [replace(row,character_failures=('attack_median_delta_db:unavailable',))
                if row.policy == 'single' else row for row in self.cases['funk']]
        selected = self.resolve(rows)
        self.assertFalse(selected.character_qualified)
        self.assertEqual(selected.reason,'character_fallback')

    def test_invalid_control_cannot_be_a_fallback(self):
        rows = [replace(row,technical_failures=('unverified_whole_file',))
                if row.policy == 'control' else row for row in self.cases['metal']]
        selected = self.resolve(rows)
        self.assertIsNone(selected.selected)
        self.assertEqual(selected.reason,'no_verified_fallback')

    def test_nonfinite_candidate_cannot_win(self):
        rows = [replace(row,lufs=float('nan')) if row.policy == 'single' else row for row in self.cases['baby']]
        self.assertEqual(self.resolve(rows).selected.id,'baby-control')

    def test_mixed_context_or_duplicate_id_is_rejected(self):
        rows = self.cases['baby']
        with self.assertRaises(AssertionError):
            self.resolve([rows[0],replace(rows[1],context='different-source-or-settings')])
        with self.assertRaises(AssertionError):
            self.resolve([rows[0],replace(rows[1],id=rows[0].id)])

    def test_result_is_immutable(self):
        result = self.resolve(self.cases['baby'])
        with self.assertRaises(FrozenInstanceError):
            result.reason = 'invented'


if __name__ == '__main__':
    unittest.main()
