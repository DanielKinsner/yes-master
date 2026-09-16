"""Regression checks for source anchors, paired statistics and explicit fallback."""
import copy
import tempfile
import unittest
from pathlib import Path
import numpy as np
import soundfile as sf
from drive_metrics import measure, compare, constraints
from evaluate_drive import select


class DriveMeasurements(unittest.TestCase):
    def test_common_gain_retains_original_windows_and_character(self):
        rate = 48000
        x = np.random.default_rng(761).normal(size=(rate * 30, 2)) * .025
        x[rate * 10:rate * 20] *= 3
        with tempfile.TemporaryDirectory() as tmp:
            source_path = Path(tmp) / 'source.wav'
            copy_path = Path(tmp) / 'copy.wav'
            sf.write(source_path, x, rate, subtype='FLOAT')
            sf.write(copy_path, x * .0001, rate, subtype='FLOAT')
            source = measure(source_path)
            quiet = measure(copy_path, source['anchors'])
        self.assertEqual(quiet['anchors'], source['anchors'])
        self.assertEqual(len(source['anchors']['active_section_indices']), 3)
        self.assertTrue(all(x < -40 for x in quiet['sections10_rms_db']))
        delta = compare(source, quiet)
        self.assertLess(abs(delta['section_contrast_delta_db']), 1e-5)
        self.assertLess(max(abs(v) for v in delta['paired_attack_delta_db']), 1e-5)
        self.assertLess(max(abs(v) for v in delta['band_delta_db']), 1e-5)
        self.assertEqual(constraints(source, delta, compare(source, source)), [])

    def test_attack_percentiles_are_paired_before_aggregation(self):
        source = dict(spectral_rms_db=-20, sections10_rms_db=[-20, -10],
                      anchors={'section_indices': [0, 1]},
                      transient_attack_body_db=[0, 10, 11], bands_db=[-20] * 9, stereo=None)
        candidate = copy.deepcopy(source)
        candidate['transient_attack_body_db'] = [5, 0, 17]
        delta = compare(source, candidate)
        self.assertEqual(delta['paired_attack_delta_db'], [5, -10, 6])
        self.assertEqual(delta['attack_median_delta_db'], 5)

    def test_missing_required_character_data_cannot_be_feasible(self):
        source = dict(channels=1)
        delta = dict(section_contrast_delta_db=None, attack_median_delta_db=None,
                     attack_p10_delta_db=None, band_delta_db=[0] * 9, stereo_delta=None)
        self.assertEqual(len(constraints(source, delta, delta)), 3)
        control = dict(id='corrected-control', technical_pass=True,
                       target_error_lu=-2., character_failures=['section:unavailable'])
        selection = select([], True, control)
        self.assertTrue(selection['fallback'])
        self.assertEqual(selection['selected'], 'corrected-control')
        self.assertEqual(selection['target_error_lu'], -2.)

    def test_target_hit_preference_preserves_rule_nearness(self):
        rows = [dict(id='near', technical_pass=True, target_error_lu=.1,
                     character_failures=[], offset_db=0),
                dict(id='far', technical_pass=True, target_error_lu=0.,
                     character_failures=[], offset_db=-6)]
        self.assertEqual(select(rows, True, rows[0])['selected'], 'near')

    def test_refinement_rechecks_each_midpoint_without_assuming_feasibility(self):
        from refine_drive import next_refinement
        def row(offset, error, qualified):
            return dict(id=str(offset), policy='bounded', technical_pass=True,
                        offset_db=offset, target_error_lu=error,
                        character_failures=[] if qualified else ['section'])
        rows = [row(-6, -3, True), row(-3, -1.5, True), row(0, 0, False)]
        self.assertEqual(next_refinement(rows), -1.5)
        rows.append(row(-1.5, -.5, False))
        self.assertEqual(next_refinement(rows), -2.25)
        rows[-1] = row(-1.5, -.7, True)
        self.assertEqual(next_refinement(rows), -.75)

    def test_refinement_stops_for_a_qualified_hit_or_missing_feasible_anchor(self):
        from refine_drive import next_refinement
        row = dict(id='only', policy='single', technical_pass=True,
                   offset_db=0, target_error_lu=0, character_failures=[])
        self.assertIsNone(next_refinement([row]))
        row['character_failures'] = ['section']
        self.assertIsNone(next_refinement([row]))


if __name__ == '__main__':
    unittest.main()
