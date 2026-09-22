"""Regression checks for the new diagnostic's eligibility and fixed tie order."""
import unittest
from reference_headroom_research import normalized_pool, copy_check


class DiagnosticTests(unittest.TestCase):
    def pool(self):
        return [dict(rank=i, lufs=-16.-i, failures=[], technical_pass=True) for i in range(4)]

    def test_target_priority_and_fixed_tie_order(self):
        pool = self.pool()
        pool[1]['lufs'] = -14.1
        pool[3]['lufs'] = -14.
        self.assertEqual(normalized_pool(pool)['selected']['rank'], 1)
        self.assertEqual(normalized_pool(pool)['reason'], 'qualified_target')

    def test_shortfall_distance_and_ties(self):
        pool = self.pool()
        pool[3]['lufs'] = -15.
        self.assertEqual(normalized_pool(pool)['selected']['rank'], 3)
        pool[0]['lufs'] = -15.
        self.assertEqual(normalized_pool(pool)['selected']['rank'], 0)

    def test_invalid_character_and_unavailable_cannot_win(self):
        pool = self.pool()
        pool[0].update(lufs=-14., technical_pass=False)
        pool[1].update(lufs=-14., failures=['attack_p10_delta_db'])
        pool[2]['lufs'] = None
        pool[3]['lufs'] = float('nan')
        self.assertIsNone(normalized_pool(pool)['selected'])
        self.assertEqual(normalized_pool(pool)['reason'], 'no_eligible_normalized_candidate')

    def test_missing_pool_is_rejected(self):
        with self.assertRaises(AssertionError):
            normalized_pool(self.pool()[:3])

    def test_overshoot_is_not_a_shortfall(self):
        pool = self.pool()
        pool[0]['lufs'] = -13.7
        self.assertEqual(normalized_pool(pool)['selected']['rank'], 1)

    def test_copy_missing_lufs_and_excess_pcm_fail(self):
        original = dict(lufs=-20., drive_db=-10.)
        candidate = dict(lufs=-20., drive_db=-10., maximum_copy_difference=0.)
        delta = dict(section_contrast_delta_db=0., section_delta_db=[0.], paired_attack_delta_db=[0.],
            band_delta_db=[0.], stereo_delta=dict(side_mid_db=0., correlation=0.))
        self.assertTrue(copy_check(original, candidate, delta)['pass_all'])
        candidate.update(lufs=None, maximum_copy_difference=.00011)
        self.assertEqual(copy_check(original, candidate, delta)['failures'], ['pcm', 'lufs'])
        self.assertEqual(original, dict(lufs=-20., drive_db=-10.))


if __name__ == '__main__':
    unittest.main()
