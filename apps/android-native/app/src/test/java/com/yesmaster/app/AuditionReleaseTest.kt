package com.yesmaster.app

import java.util.Collections
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withContext
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * Pins the destroy-vs-measureLanding lifetime contract without loading the
 * native library. The latch models measureLandingNative: cancellation requests
 * the coroutine stop, but the blocking JNI call still returns later.
 */
class AuditionReleaseTest {

    @Test
    fun waitsForInFlightLandingBeforeDestroyingHandle() = runBlocking {
        val destroyed = Collections.synchronizedList(mutableListOf<Long>())
        val landingEnteredNative = CountDownLatch(1)
        val releaseLanding = CountDownLatch(1)

        val landingJob = launch(Dispatchers.Default + Job()) {
            withContext(Dispatchers.IO) {
                landingEnteredNative.countDown()
                releaseLanding.await()
            }
        }

        assertTrue("landing should enter native work", landingEnteredNative.await(2, TimeUnit.SECONDS))
        val destroyJob = AuditionRelease.destroyAfterLanding(
            scope = this,
            ioDispatcher = Dispatchers.Default,
            landingJob = landingJob,
            handle = 77L,
            destroy = { destroyed.add(it) },
        )
        assertNotNull("non-zero handle schedules a destroy job", destroyJob)

        Thread.sleep(50)
        assertTrue(
            "destroy must wait for the uninterruptible landing call",
            destroyed.isEmpty(),
        )

        releaseLanding.countDown()
        destroyJob!!.join()

        assertEquals(listOf(77L), destroyed.toList())
    }

    @Test
    fun zeroHandleDoesNotScheduleDestroyWork() = runBlocking {
        val destroyJob = AuditionRelease.destroyAfterLanding(
            scope = this,
            ioDispatcher = Dispatchers.Default,
            landingJob = null,
            handle = 0L,
            destroy = { error("zero handle must not be destroyed") },
        )
        assertNull(destroyJob)
    }
}
