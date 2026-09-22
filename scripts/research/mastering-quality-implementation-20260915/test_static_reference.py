"""Reference component identities, channel behavior and numerical comparisons."""
import unittest
import numpy as np
from static_reference_research import independent_reference, relative_error


def topology(active=False,width=1.):
    identity=[1.,0.,0.,0.,0.]
    return dict(eq=[identity],compression_active=active,branches=[[identity]]*3,
                makeup=[1.,2.,3.],width=width)


class StaticReferenceTests(unittest.TestCase):
    def test_identity_preserves_samples_and_input(self):
        samples=np.array([[.25,-.5],[0.,.125]],dtype=np.float32)
        before=samples.copy()
        np.testing.assert_array_equal(independent_reference(samples,topology()),samples)
        np.testing.assert_array_equal(samples,before)

    def test_zero_dynamic_reduction_still_retains_makeup_and_branch_sum(self):
        samples=np.array([[.125,-.25],[0.,.0625]],dtype=np.float32)
        np.testing.assert_array_equal(independent_reference(samples,topology(True)),samples*6)
        self.assertGreater(relative_error(samples,independent_reference(samples,topology(True))),1.)

    def test_width_preserves_mid_and_changes_side(self):
        source=np.array([[1.,0.],[.5,.5]],dtype=np.float32)
        expected=np.array([[1.5,-.5],[.5,.5]],dtype=np.float32)
        np.testing.assert_array_equal(independent_reference(source,topology(width=2.)),expected)
        np.testing.assert_array_equal(independent_reference(source,topology(width=0.)),np.array([[.5,.5],[.5,.5]],dtype=np.float32))

    def test_comparison_sees_late_channel_error(self):
        a=np.ones((300000,2),dtype=np.float32);b=a.copy();b[-1,1]=1.01
        self.assertGreater(relative_error(a,b),.009)


if __name__=='__main__':
    unittest.main()
