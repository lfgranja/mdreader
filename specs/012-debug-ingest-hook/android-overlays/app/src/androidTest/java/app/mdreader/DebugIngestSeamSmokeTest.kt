package app.mdreader

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import org.junit.Assert.assertEquals
import org.junit.Test
import org.junit.runner.RunWith

/**
 * Canonical FR-013/FR-011 androidTest smoke scaffold for the debug ingest seam.
 * Exercises debug-only automation contract shape only (DebugIngestActivity /
 * app.mdreader.action.DEBUG_INGEST intent contract); no manual SAF picker/UI traversal.
 * Materialized into src-tauri/gen/android/app/src/androidTest/ by FR-011 apply/check
 * only when this canonical source exists.
 *
 * B1 instrumented evidence vocabulary (never imply execution from scaffolding):
 * scaffolded | materialized | executed-pass | executed-fail | blocked:<reason>.
 */
@RunWith(AndroidJUnit4::class)
class DebugIngestSeamSmokeTest {
    @Test
    fun debugPackageIsTargeted() {
        val appContext = InstrumentationRegistry.getInstrumentation().targetContext
        assertEquals("app.mdreader", appContext.packageName)
    }
}
