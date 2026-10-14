package app.mdreader

import android.app.Activity
import android.content.ContentResolver
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Build
import android.util.Log
import java.io.File
import java.io.FileOutputStream
import java.io.InputStream
import java.security.MessageDigest

/**
 * Document ingestion: picker → single read → snapshot → BLAKE3 fingerprint.
 *
 * Implements contracts/ingest.md flow:
 * 1. ACTION_OPEN_DOCUMENT with text/markdown + text/plain filter
 * 2. takePersistableUriPermission() best-effort in try/catch(SecurityException)
 * 3. ContentResolver.openInputStream(uri) → read bytes once
 * 4. fingerprint = blake3_256(bytes); write filesDir/snapshots/<hex>.{md,txt}
 * 5. Trim all but current snapshot on new pick / Stop / document-end
 *
 * Zero position logic: Kotlin owns intent + stream + snapshot write;
 * Rust owns hash/parse/lookup/synthesis.
 */
object DocIngest {

    private const val TAG = "DocIngest"

    /** Request code for ACTION_OPEN_DOCUMENT. */
    private const val PICK_DOCUMENT_REQUEST = 0x4D4F // "MO" as u16

    /** Snapshot directory under filesDir (not cache — survives app restarts). */
    const val SNAPSHOT_DIR = "snapshots"

    /** MIME types accepted by the picker. */
    private val ACCEPTED_MIME_TYPES = arrayOf("text/markdown", "text/plain")

    /** Extension-to-MIME mapping for snapshot file naming. */
    private val EXTENSION_BY_MIME = mapOf(
        "text/markdown" to "md",
        "text/plain" to "txt"
    )

    /**
     * Launches the system document picker for markdown/plain text files.
     *
     * @param activity The Activity to start the picker from (for startActivityForResult).
     * @return true if picker was launched, false if no activity or picker unavailable.
     */
    fun pickDocument(activity: Activity): Boolean {
        val intent = Intent(Intent.ACTION_OPEN_DOCUMENT).apply {
            addCategory(Intent.CATEGORY_OPENABLE)
            type = "*/*" // We filter by MIME in putExtra
            putExtra(Intent.EXTRA_MIME_TYPES, ACCEPTED_MIME_TYPES)
            putExtra(Intent.EXTRA_ALLOW_MULTIPLE, false)
            flags = Intent.FLAG_GRANT_READ_URI_PERMISSION or Intent.FLAG_GRANT_PERSISTABLE_URI_PERMISSION
        }

        // Verify there's an activity to handle the intent
        val resolveInfo = activity.packageManager.queryIntentActivities(intent, 0)
        if (resolveInfo.isEmpty()) {
            Log.w(TAG, "No activity handles ACTION_OPEN_DOCUMENT for text/markdown|text/plain")
            return false
        }

        try {
            activity.startActivityForResult(intent, PICK_DOCUMENT_REQUEST)
            return true
        } catch (e: Exception) {
            Log.e(TAG, "Failed to start document picker", e)
            return false
        }
    }

    /**
     * Processes the picker result from onActivityResult.
     *
     * @param activity The Activity that received the result.
     * @param requestCode The request code (must match PICK_DOCUMENT_REQUEST).
     * @param resultCode The result code (Activity.RESULT_OK expected).
     * @param data The result Intent containing the picked document URI.
     * @return IngestResult with snapshot path and fingerprint, or null on failure/cancel.
     */
    fun handlePickResult(
        activity: Activity,
        requestCode: Int,
        resultCode: Int,
        data: Intent?
    ): IngestResult? {
        if (requestCode != PICK_DOCUMENT_REQUEST) {
            return null // Not our request
        }

        if (resultCode != Activity.RESULT_OK || data == null || data.data == null) {
            Log.d(TAG, "Document pick cancelled or failed: resultCode=$resultCode")
            return null
        }

        val uri = data.data!!
        return try {
            // Best-effort persistable grant (failure still allows one-shot read)
            takePersistableUriPermission(activity.contentResolver, uri)

            // Single read via ContentResolver
            val bytes = readDocumentBytes(activity.contentResolver, uri)

            // Compute BLAKE3 fingerprint
            val fingerprint = blake3_256(bytes)

            // Write snapshot to filesDir/snapshots/<hex>.{md,txt}
            val snapshotFile = writeSnapshot(activity, uri, bytes, fingerprint)

            // Trim all other snapshots (keep-current-only)
            trimSnapshots(activity, snapshotFile)

            IngestResult(
                snapshotPath = snapshotFile.absolutePath,
                fingerprintHex = bytesToHex(fingerprint),
                originalUri = uri.toString(),
                mimeType = activity.contentResolver.getType(uri) ?: "text/plain"
            ).also { result ->
                // Forward to Rust for resume-offer lookup and playback
                onDocumentIngested(
                    result.snapshotPath,
                    result.fingerprintHex,
                    result.originalUri,
                    result.mimeType
                )
            }
        } catch (e: SecurityException) {
            Log.w(TAG, "Persistable grant failed; one-shot read will be used next time", e)
            // Fall through: try to read with one-shot grant
            try {
                val bytes = readDocumentBytes(activity.contentResolver, uri)
                val fingerprint = blake3_256(bytes)
                val snapshotFile = writeSnapshot(activity, uri, bytes, fingerprint)
                trimSnapshots(activity, snapshotFile)
                IngestResult(
                    snapshotPath = snapshotFile.absolutePath,
                    fingerprintHex = bytesToHex(fingerprint),
                    originalUri = uri.toString(),
                    mimeType = activity.contentResolver.getType(uri) ?: "text/plain"
                ).also { result ->
                    // Forward to Rust for resume-offer lookup and playback
                    onDocumentIngested(
                        result.snapshotPath,
                        result.fingerprintHex,
                        result.originalUri,
                        result.mimeType
                    )
                }
            } catch (e2: Exception) {
                Log.e(TAG, "One-shot read also failed", e2)
                null
            }
        } catch (e: Exception) {
            Log.e(TAG, "Document ingest failed", e)
            null
        }
    }

    /**
     * Best-effort persistable URI permission grant.
     * Failure is non-fatal — one-shot grant still allows the read.
     */
    private fun takePersistableUriPermission(resolver: ContentResolver, uri: Uri) {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.KITKAT) {
            try {
                val takeFlags = Intent.FLAG_GRANT_READ_URI_PERMISSION
                resolver.takePersistableUriPermission(uri, takeFlags)
                Log.d(TAG, "Persistable URI permission granted")
            } catch (e: SecurityException) {
                // Expected on some devices/launchers — re-throw for caller to handle fallback
                throw e
            }
        }
    }

    /**
     * Reads document bytes once via ContentResolver.
     * Single read per contract — no streaming, no partial reads.
     */
    private fun readDocumentBytes(resolver: ContentResolver, uri: Uri): ByteArray {
        val inputStream: InputStream = resolver.openInputStream(uri)
            ?: throw java.io.FileNotFoundException("ContentResolver returned null stream for $uri")

        return try {
            inputStream.readAllBytes()
        } finally {
            inputStream.close()
        }
    }

    /**
     * Computes BLAKE3-256 hash of document bytes.
     * Pure function of content — matches Rust `session::fingerprint`.
     */
    private fun blake3_256(data: ByteArray): ByteArray {
        // Delegate to Rust via JNI for exact BLAKE3-256 match
        return Blake3Hasher.hash(data)
    }

    /**
     * Writes snapshot file to filesDir/snapshots/<fingerprint-hex>.{md,txt}.
     * Extension derived from original MIME type.
     */
    private fun writeSnapshot(
        context: Context,
        uri: Uri,
        bytes: ByteArray,
        fingerprint: ByteArray
    ): File {
        val snapshotsDir = File(context.filesDir, SNAPSHOT_DIR)
        if (!snapshotsDir.exists() && !snapshotsDir.mkdirs()) {
            throw IllegalStateException("Failed to create snapshots directory: $snapshotsDir")
        }

        val mimeType = context.contentResolver.getType(uri) ?: "text/plain"
        val extension = EXTENSION_BY_MIME[mimeType] ?: "txt"
        val hexName = bytesToHex(fingerprint)
        val snapshotFile = File(snapshotsDir, "$hexName.$extension")

        FileOutputStream(snapshotFile).use { fos ->
            fos.write(bytes)
        }

        Log.d(TAG, "Snapshot written: ${snapshotFile.absolutePath} (${bytes.size} bytes)")
        return snapshotFile
    }

    /**
     * Trims all snapshots except the current one (keep-current-only policy).
     * Called on new pick, Stop, and document-end.
     */
    fun trimSnapshots(context: Context, keep: File) {
        val snapshotsDir = File(context.filesDir, SNAPSHOT_DIR)
        if (!snapshotsDir.exists()) return

        snapshotsDir.listFiles()?.forEach { file ->
            if (file != keep && file.isFile) {
                val deleted = file.delete()
                if (deleted) {
                    Log.d(TAG, "Trimmed old snapshot: ${file.name}")
                } else {
                    Log.w(TAG, "Failed to trim snapshot: ${file.name}")
                }
            }
        }
    }

    /**
     * Clears all snapshots (Stop / document-end path).
     */
    fun clearAllSnapshots(context: Context) {
        val snapshotsDir = File(context.filesDir, SNAPSHOT_DIR)
        if (!snapshotsDir.exists()) return

        snapshotsDir.listFiles()?.forEach { file ->
            if (file.isFile) {
                val deleted = file.delete()
                if (deleted) {
                    Log.d(TAG, "Cleared snapshot: ${file.name}")
                }
            }
        }
    }

    /**
     * Converts byte array to lowercase hex string.
     */
    private fun bytesToHex(bytes: ByteArray): String {
        val sb = StringBuilder(bytes.size * 2)
        for (b in bytes) {
            sb.append(String.format("%02x", b))
        }
        return sb.toString()
    }

    /**
     * Result of document ingestion: snapshot path + fingerprint + metadata.
     */
    data class IngestResult(
        val snapshotPath: String,
        val fingerprintHex: String,
        val originalUri: String,
        val mimeType: String
    )

    /**
     * Native callback to Rust after successful document ingestion.
     * Forwards snapshot path, fingerprint, original URI, and MIME type.
     */
    private external fun onDocumentIngested(
        snapshotPath: String,
        fingerprintHex: String,
        originalUri: String,
        mimeType: String
    )

    /**
     * BLAKE3-256 hasher — delegates to Rust via JNI for exact match
     * with `session::fingerprint::fingerprint`.
     */
    private object Blake3Hasher {
        // Simplified: delegate to Rust via JNI
        fun hash(data: ByteArray): ByteArray {
            return try {
                // Call Rust via JNI for accurate BLAKE3-256 hash
                nativeBlake3(data)
            } catch (e: UnsatisfiedLinkError) {
                // Fallback: use SHA-256 for development (NOT cryptographically equivalent)
                Log.w(TAG, "Native BLAKE3 not available, using SHA-256 fallback")
                fallbackHash(data)
            }
        }

        @Suppress("UNUSED_PARAMETER")
        private external fun nativeBlake3(data: ByteArray): ByteArray

        // Simple fallback hash (NOT BLAKE3 - for development only)
        private fun fallbackHash(data: ByteArray): ByteArray {
            val md = MessageDigest.getInstance("SHA-256")
            return md.digest(data)
        }
    }
}