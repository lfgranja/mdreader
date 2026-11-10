package app.mdreader

import android.app.Activity
import android.content.Intent
import android.net.Uri
import android.os.Bundle
import android.util.Log

/**
 * Debug-only automation entrypoint (exported only in debug manifest).
 * Accepts adb intents and routes to the same ingest callbacks as SAF flows.
 *
 * Examples:
 *   adb shell am start -n app.mdreader/.DebugIngestActivity --es mode document --es path /sdcard/Download/mdreader-test-fixtures/t184-teste.md
 *   adb shell am start -n app.mdreader/.DebugIngestActivity --es mode directory --es path /sdcard/Download/mdreader-test-fixtures/md-dir
 *   adb shell am start -n app.mdreader/.DebugIngestActivity -a app.mdreader.action.DEBUG_INGEST --es mode uri -d content://...
 */
class DebugIngestActivity : Activity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val result = try {
            if (!BuildConfig.DEBUG) {
                "refused:not-debug"
            } else {
                handle(intent)
            }
        } catch (e: Exception) {
            Log.e(TAG, "DebugIngestActivity failed", e)
            "error:${e.javaClass.simpleName}:${e.message}"
        }
        Log.i(TAG, "debug ingest result: $result")
        try {
            val out = java.io.File(applicationContext.cacheDir, "debug_ingest_result.txt")
            out.writeText(result)
        } catch (_: Exception) {}
        setResult(RESULT_OK, Intent().putExtra(EXTRA_RESULT, result))
        finish()
    }

    private fun handle(intent: Intent?): String {
        val mode = intent?.getStringExtra(EXTRA_MODE)?.lowercase()
            ?: intent?.action?.substringAfterLast('.')?.lowercase()
            ?: "document"
        val path = intent?.getStringExtra(EXTRA_PATH)
        val uri = intent?.data ?: intent?.getStringExtra(EXTRA_URI)?.let(Uri::parse)
        val ctx = applicationContext
        return when (mode) {
            "document", "file", "open_document" -> when {
                path != null -> DebugIngestHook.ingestFile(ctx, path)
                uri != null && uri.scheme == "file" -> DebugIngestHook.ingestFile(ctx, uri.path ?: "")
                uri != null -> DebugIngestHook.ingestUri(ctx, uri.toString())
                else -> "error:missing-path-or-uri"
            }
            "directory", "dir", "tree", "open_directory" -> when {
                path != null -> DebugIngestHook.ingestDirectory(ctx, path)
                uri != null && uri.scheme == "file" -> DebugIngestHook.ingestDirectory(ctx, uri.path ?: "")
                else -> "error:missing-directory-path"
            }
            "pick_document", "pick", "open_picker", "picker" -> {
                // Automation aid (debug only, gated at onCreate): exercises the same
                // ShellHost -> DocIngest SAF launch path as the title-box action.
                if (ShellHost.pickDocument()) "ok:pick_document_launched" else "error:pick_document_false"
            }
            "pick_directory" -> {
                if (ShellHost.pickDirectory()) "ok:pick_directory_launched" else "error:pick_directory_false"
            }
            "uri" -> if (uri != null) DebugIngestHook.ingestUri(ctx, uri.toString()) else "error:missing-uri"
            else -> "error:unknown-mode:$mode"
        }
    }

    companion object {
        private const val TAG = "DebugIngestActivity"
        const val EXTRA_MODE = "mode"
        const val EXTRA_PATH = "path"
        const val EXTRA_URI = "uri"
        const val EXTRA_RESULT = "result"
    }
}
