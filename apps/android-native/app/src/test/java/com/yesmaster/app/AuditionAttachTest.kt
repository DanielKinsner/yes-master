package com.yesmaster.app

import java.util.Collections
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.cancel
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * Pins the ownership contract of [AuditionAttach.createOrRelease] — the one
 * place an audition handle can leak. Runs on the JVM lane with plain lambdas
 * standing in for the JNI create/destroy, so no native library or Android
 * Context is needed. The latch dance models reality: createNative is an
 * uninterruptible whole-file decode, so a coroutine cancelled mid-decode still
 * sees the call complete and hand back a live handle.
 */
class AuditionAttachTest {

    @Test
    fun returnsTheHandleToTheCallerWhenStillActive() = runBlocking {
        val destroyed = mutableListOf<Long>()

        val handle = AuditionAttach.createOrRelease(
            ioDispatcher = Dispatchers.IO,
            create = { 7L },
            destroy = { destroyed.add(it) },
        )

        assertEquals("the active attach owns the handle", 7L, handle)
        assertTrue("an owned handle must not be freed under us", destroyed.isEmpty())
    }

    @Test
    fun destroysAHandleOrphanedByCancellationDuringTheDecode() = runBlocking {
        val created = Collections.synchronizedList(mutableListOf<Long>())
        val destroyed = Collections.synchronizedList(mutableListOf<Long>())
        val decodeStarted = CountDownLatch(1)
        val releaseDecode = CountDownLatch(1)

        // A separate scope so we can cancel it the way attach()'s release()
        // cancels the previous prepareJob. Default dispatcher (not IO) so the
        // withContext(IO) hop must redispatch — exactly the production setup,
        // where the controller runs on the Main/viewModelScope dispatcher.
        val scope = CoroutineScope(Dispatchers.Default + Job())
        val job = scope.launch {
            AuditionAttach.createOrRelease(
                ioDispatcher = Dispatchers.IO,
                create = {
                    val handle = 42L
                    created.add(handle)
                    decodeStarted.countDown()
                    // Block like an uninterruptible JNI decode; coroutine
                    // cancellation cannot interrupt this.
                    releaseDecode.await()
                    handle
                },
                destroy = { destroyed.add(it) },
            )
        }

        assertTrue("decode should start", decodeStarted.await(2, TimeUnit.SECONDS))
        // A different track is attached while we're still decoding.
        job.cancel()
        // Now the decode finishes and returns a live handle into a coroutine
        // that is already cancelled.
        releaseDecode.countDown()
        job.join()

        assertEquals("one handle was created", listOf(42L), created.toList())
        assertEquals(
            "the orphaned handle must be destroyed, not leaked",
            created.toSet(),
            destroyed.toSet(),
        )
        scope.cancel()
    }
}
