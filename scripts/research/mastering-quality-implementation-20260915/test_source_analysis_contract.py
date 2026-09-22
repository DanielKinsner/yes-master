import unittest
from source_analysis_contract import profile_comparison, coefficient_comparison


class ContractAuditTests(unittest.TestCase):
    def test_missing_profile_is_not_equivalence(self):
        self.assertFalse(profile_comparison(None,None)['pass_all'])
        self.assertFalse(profile_comparison({'stereo_correlation':None},{'stereo_correlation':None})['pass_all'])

    def test_gate_range_drift_fails_without_masking_stable_spectrum(self):
        a = dict(dynamic_range_lu=8., spectral_6={'air':.1})
        b = dict(dynamic_range_lu=0., spectral_6={'air':.1000001})
        result = profile_comparison(a,b)
        self.assertEqual(result['failures'],['dynamic_range_lu'])
        self.assertEqual(result['differences']['dynamic_range_lu'],8.)

    def test_coefficients_separate_structure_numeric_and_exact_identity(self):
        a = 'Chain { b0: 1.0, active: false, gain: 1e-5 }'
        b = 'Chain { b0: 1.00001, active: false, gain: 1e-5 }'
        result = coefficient_comparison(a,b)
        self.assertTrue(result['structure_identity'])
        self.assertFalse(result['exact_identity'])
        self.assertAlmostEqual(result['maximum_absolute_numeric_difference'],.00001)
        self.assertFalse(coefficient_comparison(a,b.replace('false','true'))['structure_identity'])
        self.assertFalse(coefficient_comparison('gain: NaN','gain: NaN')['numeric_comparison_available'])


if __name__ == '__main__':
    unittest.main()
