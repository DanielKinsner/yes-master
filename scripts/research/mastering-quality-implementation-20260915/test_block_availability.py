"""Failure-oriented parser and native-state regressions for the new version."""
import unittest
from block_availability import classify, agreement


def trace(momentary=-80., integrated=-70., threshold=0.):
    lines=[f't: {(i*4800-1)/48000:.6f} TARGET:-23 LUFS M: {momentary:.1f} S:-90.0'
           for i in range(1,21)]
    lines.append(f'Summary:\n Integrated loudness:\n I: {integrated:.1f} LUFS\n Threshold: {threshold:.1f} LUFS\n Loudness range:')
    return '\n'.join(lines)


class BlockAvailabilityTests(unittest.TestCase):
    def test_no_blocks_and_fabricated_floor(self):
        result=classify(trace(),96000)
        self.assertEqual(result['state'],'unavailable')
        self.assertIsNone(result['lufs'])
        self.assertTrue(agreement('unavailable',None,result))
        self.assertFalse(agreement('finite',-70.,result))

    def test_finite_floor_display_needs_positive_block_evidence(self):
        result=classify(trace(-70.,-70.,-80.),96000)
        self.assertEqual(result['state'],'finite')
        self.assertTrue(agreement('finite',-69.96,result))
        self.assertFalse(agreement('unavailable',None,result))

    def test_rounded_boundary_remains_unknown(self):
        result=classify(trace(-70.),96000)
        self.assertEqual(result['state'],'unknown')
        self.assertIsNone(result['lufs'])
        self.assertFalse(agreement('unavailable',None,result))

    def test_missing_duplicate_reordered_or_malformed_trace_is_unknown(self):
        lines=trace().splitlines()
        cases=[lines[1:],lines[:8]+lines[9:],lines[:8]+[lines[7]]+lines[8:],
               [lines[1],lines[0]]+lines[2:]]
        for case in cases:
            self.assertEqual(classify('\n'.join(case),96000)['state'],'unknown')
        for old,new in [('M: -80.0','M: nan'),('TARGET:-23 LUFS','TARGET:-23 LU'),
                        ('t: 0.099979','t: 0.199979')]:
            self.assertEqual(classify(trace().replace(old,new,1),96000)['state'],'unknown')

    def test_incomplete_or_duplicate_summary_is_unknown(self):
        for log in [trace().split('Summary:')[0],trace()+'\nSummary:',trace().replace('Threshold: 0.0','Threshold: nan')]:
            self.assertEqual(classify(log,96000)['state'],'unknown')

    def test_unsupported_input_is_unknown(self):
        for frames,rate,channels in [(14400,48000,2),(96000,44100,2),(96000,48000,1)]:
            self.assertEqual(classify(trace(),frames,rate,channels)['state'],'unknown')

    def test_native_errors_and_mislabeled_nonfinite_never_agree(self):
        result=classify(trace(),96000)
        for state,value in [('error',None),('finite',float('nan')),('finite',float('inf')),
                            ('finite',-float('inf')),('unavailable',-70.)]:
            self.assertFalse(agreement(state,value,result))


if __name__=='__main__':
    unittest.main()
