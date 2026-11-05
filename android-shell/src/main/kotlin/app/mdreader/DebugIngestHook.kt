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

    fun isEnabled(): Boolean = BuildConfig.DEBUG

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
        } catch (e: Exception) {
            Log.e(TAG, "debug ingestFile failed", e)
            "error:${e.javaClass.simpleName}:${e.message}"
        }
    }

    @JvmStatic
    fun ingestUri(context: Context, uriString: String): String {
        if (!BuildConfig.DEBUG) return "refused:not-debug"
        return try {
            val uri = Uri.parse(uriString)
            val resolver = context.contentResolver
            val bytes = resolver.openInputStream(uri)?.use { it.readBytes() }
                ?: return "error:unreadable-uri"
            val name = uri.lastPathSegment ?: "document"
            val extension = name.substringAfterLast('.', "").lowercase().ifEmpty { "txt" }
            if (extension !in setOf("md", "txt")) return "error:unsupported-extension:$extension"
            if (bytes.size > DocIngest.MAX_DOC_BYTES) {
                return "refused:${DocIngest.REFUSAL_FORMAT_UNDELIVERABLE}:size"
            }
            DocIngest.debugDeliverDocument(
                context,
                bytes,
                extension,
                uri.toString(),
                if (extension == "md") "text/markdown" else "text/plain",
                name
            )
            "ok:uri:$uriString"
        } catch (e: Exception) {
            Log.e(TAG, "debug ingestUri failed", e)
            "error:${e.javaClass.simpleName}:${e.message}"
        }
    }

    @JvmStatic
    fun ingestDirectory(context: Context, dirPath: String): String {
        if (!BuildConfig.DEBUG) return "refused:not-debug"
        val root = File(dirPath)
        if (!root.isDirectory) return "error:not-a-directory:$dirPath"
        return try {
            val chapters = JSONArray()
            fun addFile(file: File, relativePath: String) {
                val ext = file.extension.lowercase()
                if (ext !in setOf("md", "txt")) return
                val bytes = file.readBytes()
                if (bytes.size > DocIngest.MAX_DOC_BYTES) return
                val hex = DocIngest.debugFingerprintHex(bytes)
                val snapshot = DocIngest.debugWriteSnapshot(context, bytes, ext, hex)
                chapters.put(
                    JSONObject()
                        .put("filename", file.name)
                        .put("snapshot_path", snapshot.absolutePath)
                        .put("relative_path", relativePath)
                        .put("byte_size", bytes.size)
                        .put("fingerprint_hex", hex)
                )
                if (chapters.length() == 1) {
                    DocIngest.debugDispatchDirectory(root.toURI().toString(), root.name, chapters)
                }
            }
            val rootFiles = root.listFiles()?.filter { it.isFile }?.sortedWith { a, b -> DocIngest.naturalCompare(a.name, b.name) } ?: emptyList()
            val rootDirs = root.listFiles()?.filter { it.isDirectory }?.sortedWith { a, b -> DocIngest.naturalCompare(a.name, b.name) } ?: emptyList()
            rootFiles.forEach { addFile(it, it.name) }
            rootDirs.forEach { dir ->
                dir.listFiles()?.filter { it.isFile }?.sortedWith { a, b -> DocIngest.naturalCompare(a.name, b.name) }?.forEach {
                    addFile(it, "${dir.name}/${it.name}")
                }
            }
            if (chapters.length() == 0) return "error:no-chapters"
            DocIngest.debugDispatchDirectory(root.toURI().toString(), root.name, chapters)
            "ok:directory:${root.absolutePath}:chapters=${chapters.length()}"
        } catch (e: Exception) {
            Log.e(TAG, "debug ingestDirectory failed", e)
            "error:${e.javaClass.simpleName}:${e.message}"
        }
    }
}
