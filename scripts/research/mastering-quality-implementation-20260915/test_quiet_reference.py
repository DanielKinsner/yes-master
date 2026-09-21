"""Qualify availability parsing with actual controlled FFmpeg meter results."""
import math
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

import numpy as np
import soundfile as sf
from quiet_reference import loudness_summary, loudness_agreement


class QuietReferenceTests(unittest.TestCase):
    def test_real_silent_quiet_and_finite_programmes(self):
        ffmpeg = shutil.which('ffmpeg')
        self.assertIsNotNone(ffmpeg)
        with tempfile.TemporaryDirectory(prefix='yes-master-quiet-reference-') as directory:
            t=np.arange(96000)/48000
            for name, amplitude, native in [('silent',0.,None),('quiet',10**(-90/20),None),('finite',.1,-20.)]:
                samples=np.repeat((amplitude*np.sin(2*np.pi*1000*t))[:,None],2,axis=1)
                path=Path(directory)/(name+'.wav')
                sf.write(path,samples,48000,subtype='FLOAT')
                proc=subprocess.run([ffmpeg,'-hide_banner','-nostats','-i',str(path),'-af','ebur128=peak=true',
                                     '-f','null','-'],capture_output=True,text=True,check=True)
                value,threshold=loudness_summary(proc.stderr)
                peak=20*math.log10(amplitude) if amplitude else -math.inf
                result=loudness_agreement(native,value,threshold,peak)
                self.assertTrue(result['lufs_pass'],(name,result,value,threshold))
                self.assertEqual(result['integrated_loudness_available'],native is not None)
                print(name,value,threshold,result['loudness_comparison'])
                mismatch=loudness_agreement(-70. if native is None else None,value,threshold,peak)
                self.assertFalse(mismatch['lufs_pass'])

    def test_floor_display_is_not_a_loudness_measurement(self):
        self.assertFalse(loudness_agreement(-70.,-70.,0.,-90.)['lufs_pass'])
        self.assertFalse(loudness_agreement(None,-70.,0.,-.5)['lufs_pass'])
        self.assertFalse(loudness_agreement(-14.,-14.2,-24.,-1.)['lufs_pass'])


if __name__ == '__main__':
    unittest.main()
