package app.mdreader

import android.content.Context
import android.net.Uri
import android.util.Log
import java.io.File
import org.json.JSONArray
import org.json.JSONObject

/**
 * Debug/test-only ingestion hooks (automation). Compiled into all builds
 * via android-shell source set, but every entry point is guarded by
 * [BuildConfig.DEBUG] and refuses in release — no test/debug capability
 * ships in production (Android security: test/debug features in release
 * builds are a risk).
 *
 * Routes through the same DocIngest snapshot/fingerprint + JNI callbacks
 * used by SAF picker results ([DocIngest.onDocumentIngested] /
 * [DocIngest.onDirectorySelected]) so automation exercises the same
 * Rust ingest/open flow.
 */
object DebugIngestHook {
    private const val TAG = "DebugIngestHook"
    @Volatile private var nativeLoaded = false

    fun isEnabled(): Boolean = BuildConfig.DEBUG

    private fun ensureNativeLoaded(): Boolean {
        if (nativeLoaded) return true
        return try {
            // Tauri Android loads the app's Rust library; name follows the package/lib.
            System.loadLibrary("src_tauri")
            nativeLoaded = true
            true
        } catch (e: Throwable) {
            try {
                System.loadLibrary("app_lib")
                nativeLoaded = true
                true
            } catch (_: Throwable) {
                Log.w(TAG, "Native library not loaded for debug hook", e)
                false
            }
        }
    }

    @JvmStatic
    fun ingestFile(context: Context, path: String): String {
        if (!BuildConfig.DEBUG) return "refused:not-debug"
        val file = File(path)
        if (!file.isFile) return "error:not-a-file:$path"
        val extension = file.extension.lowercase()
        if (extension !in setOf("md", "txt")) return "error:unsupported-extension:$extension"
        if (file.length() > DocIngest.MAX_DOC_BYTES.toLong()) {
            return "refused:${DocIngest.REFUSAL_FORMAT_UNDELIVERABLE}:size"
        }
        return try {
            if (!ensureNativeLoaded()) return "error:native-not-loaded"
            val bytes = file.readBytes()
            DocIngest.debugDeliverDocument(
                context,
                bytes,
                extension,
                file.toURI().toString(),
                if (extension == "md") "text/markdown" else "text/plain",
                file.name
            )
            "ok:document:${file.absolutePath}"
        } catch (e: Throwable) {
            Log.e(TAG, "debug ingestFile failed", e)
            "error:${e.javaClass.simpleName}:${e.message}"
        }
    }

    @JvmStatic
    fun ingestUri(context: Context, uriString: String): String {
        if (!BuildConfig.DEBUG) return "refused:not-debug"
        return try {
            if (!ensureNativeLoaded()) return "error:native-not-loaded"
            val uri = Uri.parse(uriString)
            if (uri.scheme == "file") {
                return ingestFile(context, uri.path ?: "")
            }
            val extension = when {
                uriString.endsWith(".md", true) -> "md"
                uriString.endsWith(".txt", true) -> "txt"
                else -> "txt"
            }
            val mime = if (extension == "md") "text/markdown" else "text/plain"
            val bytes = context.contentResolver.openInputStream(uri)?.use { it.readBytes() }
                ?: return "error:unreadable-uri:$uriString"
            if (bytes.size > DocIngest.MAX_DOC_BYTES) return "refused:${DocIngest.REFUSAL_FORMAT_UNDELIVERABLE}:size"
            DocIngest.debugDeliverDocument(context, bytes, extension, uriString, mime, uri.lastPathSegment ?: "document")
            "ok:uri:$uriString"
        } catch (e: Throwable) {
            Log.e(TAG, "debug ingestUri failed", e)
            "error:${e.javaClass.simpleName}:${e.message}"
        }
    }

    @JvmStatic
    fun ingestDirectory(context: Context, dirPath: String): String {
        if (!BuildConfig.DEBUG) return "refused:not-debug"
        val dir = File(dirPath)
        if (!dir.isDirectory) return "error:not-a-directory:$dirPath"
        return try {
            if (!ensureNativeLoaded()) return "error:native-not-loaded"
            val chapters = JSONArray()
            var ordinal = 1
            val files = dir.listFiles().orEmpty().sortedWith { a, b -> DocIngest.naturalCompare(a.name, b.name) }
            for (f in files) {
                if (!f.isFile) continue
                val ext = f.extension.lowercase()
                if (ext !in setOf("md", "txt")) continue
                val bytes = try { f.readBytes() } catch (_: Exception) { continue }
                if (bytes.isEmpty() || bytes.size > DocIngest.MAX_DOC_BYTES) continue
                if (bytes.isEmpty() || bytes.any { it.toInt() == 0 }) continue
                val hex = DocIngest.debugFingerprintHex(bytes)
                val snap = DocIngest.debugWriteSnapshot(context, bytes, ext, hex)
                chapters.put(JSONObject().apply {
                    put("ordinal", ordinal++)
                    put("sourceName", f.name)
                    put("sourceUri", f.toURI().toString())
                    put("snapshotPath", snap.absolutePath)
                    put("mimeType", if (ext == "md") "text/markdown" else "text/plain")
                    put("fingerprintHex", hex)
                })
            }
            if (chapters.length() == 0) return "error:no-chapters"
            DocIngest.debugDispatchDirectory(dir.toURI().toString(), dir.name ?: "Directory", chapters)
            "ok:directory:${dir.absolutePath}:chapters=${chapters.length()}"
        } catch (e: Throwable) {
            Log.e(TAG, "debug ingestDirectory failed", e)
            "error:${e.javaClass.simpleName}:${e.message}"
        }
    }


    @JvmStatic
    external fun debugPickDocumentViaRustJni(): String

}
