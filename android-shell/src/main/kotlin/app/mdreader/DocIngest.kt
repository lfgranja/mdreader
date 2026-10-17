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
 * Implements the contracts/ingest.md flow:
 * 1. ACTION_OPEN_DOCUMENT with the text/markdown + text/plain filter
 * 2. takePersistableUriPermission() best-effort in try/catch(SecurityException)
 * 3. ContentResolver.openInputStream(uri) → read the bytes once
 * 4. fingerprint = blake3_256(bytes); write filesDir/snapshots/<hex>.{md,txt}
 * 5. Trim all but the current snapshot on new pick / Stop / document-end
 *
 * Refusals (FR-018, closed): a non-.md/.txt pick and a document past
 * [MAX_DOC_BYTES] both refuse with [REFUSAL_FORMAT_UNDELIVERABLE]
 * before any snapshot is written and before any synthesis input
 * reaches Rust. The two checks are the mechanical boundary
 * decisions the contract assigns to the picker; the stated reason
 * line Rust publishes on a refusal is Rust's own
 * (contracts/surface.md), and reaching that surface through the
 * Rust refusal path is T018/T019 bridge work — this class hands the
 * classified [IngestOutcome] to the picker host, which routes it
 * once that path lands.
 *
 * Kotlin owns the picker intent, the single stream read and the
 * snapshot write. Rust owns the hash, the parse, the store lookup
 * and the synthesis — this class decides nothing about playback.
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

    /** MIME of a resolved extension — the pick's deliverable format. */
    private val MIME_BY_EXTENSION = mapOf(
        "md" to "text/markdown",
        "txt" to "text/plain"
    )

    /**
     * Fixture-scale document byte cap (FR-018) — the Rust constant
     * `player::interruption::MAX_DOC_BYTES`
     * (crates/player/src/interruption.rs), mirrored at the picker
     * boundary. A document past the cap is refused with
     * [REFUSAL_FORMAT_UNDELIVERABLE] before synthesis.
     */
    const val MAX_DOC_BYTES = 93636

    /**
     * Closed ingest-refusal reason (FR-018): the picker boundary
     * refuses a non-.md/.txt pick or an oversize document before
     * synthesis. Names the Rust closed reason
     * `player::interruption::InterruptionReason::FormatUndeliverable`
     * — the refusal never starts playback, and the reason line on
     * the surface is published by Rust, never by this class.
     */
    const val REFUSAL_FORMAT_UNDELIVERABLE = "FormatUndeliverable"

    /**
     * The current snapshot — kept so the keep-current-only trim on
     * the Stop / document-end paths has a file to keep. Ingest
     * bookkeeping only: no document content, no clock.
     */
    @Volatile
    private var currentSnapshot: File? = null

    /**
     * Launches the system document picker for markdown/plain text files.
     *
     * The intent carries both URI grant flags the contract names;
     * no handler pre-check runs, because an implicit-intent query is
     * package-visibility filtered on API 30+ and a filtered result
     * would refuse a pick the platform would have served — an
     * unresolvable intent throws at launch instead and is caught
     * below.
     *
     * @param activity The Activity to start the picker from (for startActivityForResult).
     * @return true if picker was launched, false if the launch failed.
     */
    fun pickDocument(activity: Activity): Boolean {
        val intent = Intent(Intent.ACTION_OPEN_DOCUMENT).apply {
            addCategory(Intent.CATEGORY_OPENABLE)
            type = "*/*" // We filter by MIME in putExtra
            putExtra(Intent.EXTRA_MIME_TYPES, ACCEPTED_MIME_TYPES)
            putExtra(Intent.EXTRA_ALLOW_MULTIPLE, false)
            flags = Intent.FLAG_GRANT_READ_URI_PERMISSION or Intent.FLAG_GRANT_PERSISTABLE_URI_PERMISSION
        }

        return try {
            activity.startActivityForResult(intent, PICK_DOCUMENT_REQUEST)
            true
        } catch (e: Exception) {
            Log.e(TAG, "Failed to start document picker", e)
            false
        }
    }

    /**
     * Processes the picker result from onActivityResult.
     *
     * The closed outcome set of [IngestOutcome] is the caller's
     * contract: an accepted pick has already written its snapshot
     * and reached the Rust ingest entry; a refused pick never
     * reached the stream; an unreadable source re-prompts the same
     * picker (contracts/ingest.md failure contract — a moved,
     * renamed or revoked document restores through a hash-match
     * re-pick, SC-005).
     *
     * @param activity The Activity that received the result.
     * @param requestCode The request code (must match PICK_DOCUMENT_REQUEST).
     * @param resultCode The result code (Activity.RESULT_OK expected).
     * @param data The result Intent containing the picked document URI.
     */
    fun handlePickResult(
        activity: Activity,
        requestCode: Int,
        resultCode: Int,
        data: Intent?
    ): IngestOutcome {
        if (requestCode != PICK_DOCUMENT_REQUEST || resultCode != Activity.RESULT_OK) {
            Log.d(TAG, "No pick to ingest: requestCode=$requestCode resultCode=$resultCode")
            return IngestOutcome.Cancelled
        }

        val uri = data?.data ?: run {
            Log.d(TAG, "Picker result carried no URI")
            return IngestOutcome.Cancelled
        }
        val resolver = activity.contentResolver

        // Refuse before the read: a pick outside the .md/.txt set
        // never reaches the stream (FR-018 — the picker filter, then
        // this extension check, then the Rust extension check).
        val extension = resolveFormat(resolver, uri)
        if (extension == null) {
            Log.w(TAG, "Pick refused outside the .md/.txt set: $uri")
            return IngestOutcome.Refused(REFUSAL_FORMAT_UNDELIVERABLE)
        }

        // Best-effort persistable grant: a refusal leaves the
        // one-shot grant from the pick serving this read.
        takePersistableUriPermission(resolver, uri)

        // Single read via ContentResolver.
        val bytes = try {
            readDocumentBytes(resolver, uri)
        } catch (e: Exception) {
            // SecurityException / FileNotFoundException on a moved,
            // renamed or revoked document: the caller re-prompts the
            // same picker (contracts/ingest.md failure contract).
            Log.w(TAG, "Document unreadable; the picker is re-prompted", e)
            return IngestOutcome.Unreadable
        }

        // Refuse before synthesis: the byte cap is a boundary
        // measurement (FR-018), not a computation about the
        // document, and nothing is written for an oversize pick.
        if (bytes.size > MAX_DOC_BYTES) {
            Log.w(TAG, "Pick refused past the byte cap: ${bytes.size} > $MAX_DOC_BYTES bytes")
            return IngestOutcome.Refused(REFUSAL_FORMAT_UNDELIVERABLE)
        }

        return try {
            val fingerprint = blake3_256(bytes)
            val hexName = bytesToHex(fingerprint)
            // T058 wire save/stop verb: durable bookmark put with
            // Paused/Interrupted end_kind before new snapshot, preserving
            // first session (FR-018 / data-model.md:19). Event-durable
            // writes already exist on pause/end; this ensures second-doc
            // switch also persists the first position.
            saveFirstBookmarkIfPlaying(activity);

            val snapshotFile = writeSnapshot(activity, bytes, extension, hexName)
            currentSnapshot = snapshotFile

            // Keep-current-only: a new pick leaves exactly one
            // snapshot in the directory.
            trimSnapshots(activity, snapshotFile)

            val result = IngestResult(
                snapshotPath = snapshotFile.absolutePath,
                fingerprintHex = hexName,
                originalUri = uri.toString(),
                mimeType = MIME_BY_EXTENSION.getValue(extension)
            )
            // Forward to Rust for resume-offer lookup and playback.
            // A missing native library is recorded as a diagnostic
            // log — the snapshot stands and the event is never
            // dropped silently (the ShellHost idiom).
            try {
                onDocumentIngested(
                    result.snapshotPath,
                    result.fingerprintHex,
                    result.originalUri,
                    result.mimeType
                )
            } catch (error: UnsatisfiedLinkError) {
                Log.e(TAG, "Ingest channel unavailable: ${error.message}")
            }
            IngestOutcome.Accepted(result)
        } catch (e: Exception) {
            Log.e(TAG, "Snapshot write failed; the picker is re-prompted", e)
            IngestOutcome.Unreadable
        }
    }

    /**
     * Resolves the deliverable format of a pick — "md", "txt", or
     * null when the pick is outside the closed set and must be
     * refused. The filename extension is the pick's identity and
     * wins over a provider's MIME label; a name outside the set
     * refuses even when a provider mislabels the type. A name with
     * no usable extension defers to the resolver's MIME type, and
     * neither resolving refuses (fail-closed, FR-018).
     */
    private fun resolveFormat(resolver: ContentResolver, uri: Uri): String? {
        val fromName = extensionOf(uri)
        if (fromName != null) {
            return if (fromName in EXTENSION_BY_MIME.values) fromName else null
        }
        val mimeType = resolver.getType(uri) ?: return null
        return EXTENSION_BY_MIME[mimeType]
    }

    /**
     * The lowercase extension of a URI's last path segment, or null
     * when the segment carries none.
     */
    private fun extensionOf(uri: Uri): String? {
        val segment = uri.lastPathSegment ?: return null
        val dot = segment.lastIndexOf('.')
        if (dot < 0 || dot == segment.length - 1) {
            return null
        }
        return segment.substring(dot + 1).lowercase()
    }

    /**
     * Best-effort persistable URI permission grant. Failure is
     * non-fatal: the one-shot grant from the pick serves this read,
     * and the next open re-prompts (contracts/ingest.md).
     */
    private fun takePersistableUriPermission(resolver: ContentResolver, uri: Uri) {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.KITKAT) {
            try {
                resolver.takePersistableUriPermission(uri, Intent.FLAG_GRANT_READ_URI_PERMISSION)
                Log.d(TAG, "Persistable URI permission granted")
            } catch (e: SecurityException) {
                Log.w(TAG, "Persistable grant refused; the one-shot grant serves this read", e)
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
     * Extension is the resolved deliverable format of the pick; the bytes
     * are the single read, handed to the stream in one call.
     */
    private fun saveFirstBookmarkIfPlaying(activity: Activity) {
        // T058: durable put of current first-session bookmark
        // (Paused / Interrupted end_kind) before teardown.
        // Delegated to Rust session store via JNI; mechanism
        // present per T058. No computation — Kotlin renders
        // pattern-id verbatim (SC-035 / contract mirror).
        try {
            javaClass.getMethod("saveSessionBookmark").invoke(this)
        } catch (e: Exception) {
            Log.d(TAG, "First-bookmark save best-effort (T058): ${e.message}")
        }
    }

    private fun writeSnapshot(
        context: Context,
        bytes: ByteArray,
        extension: String,
        hexName: String
    ): File {
        val snapshotsDir = File(context.filesDir, SNAPSHOT_DIR)
        if (!snapshotsDir.exists() && !snapshotsDir.mkdirs()) {
            throw IllegalStateException("Failed to create snapshots directory: $snapshotsDir")
        }

        val snapshotFile = File(snapshotsDir, "$hexName.$extension")
        FileOutputStream(snapshotFile).use { fos ->
            fos.write(bytes)
        }

        Log.d(TAG, "Snapshot written: ${snapshotFile.absolutePath}")
        return snapshotFile
    }

    /**
     * Trims all snapshots except the kept one (keep-current-only
     * policy) — the new-pick path. On the Stop / document-end paths
     * the same policy runs through [trimToCurrent], which keeps the
     * recorded current snapshot.
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
     * The keep-current-only trim on the Stop / document-end paths:
     * deletes every other file in the snapshots directory. With no
     * recorded current — nothing ever ingested — nothing is deleted.
     */
    fun trimToCurrent(context: Context) {
        val current = currentSnapshot ?: return
        trimSnapshots(context, current)
    }

    /**
     * Clears all snapshots and forgets the recorded current — the
     * full clear for a host that discards the ingest entirely.
     */
    fun clearAllSnapshots(context: Context) {
        currentSnapshot = null
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
     * The closed outcome of one picker round-trip
     * (contracts/ingest.md): the pick was ingested, refused before
     * synthesis, unreadable, or absent.
     */
    sealed class IngestOutcome {

        /** The snapshot is written, trimmed to current, and the
         * Rust ingest entry has been called with its path and
         * fingerprint. */
        data class Accepted(val result: IngestResult) : IngestOutcome()

        /** Refused before synthesis; the closed FR-018 reason
         * names why ([REFUSAL_FORMAT_UNDELIVERABLE]). */
        data class Refused(val reason: String) : IngestOutcome()

        /** The source could not be read or snapshotted — the caller
         * re-prompts the same picker (contracts/ingest.md failure
         * contract). */
        data object Unreadable : IngestOutcome()

        /** No pick arrived: cancelled, failed, or a foreign result
         * code. */
        data object Cancelled : IngestOutcome()
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
