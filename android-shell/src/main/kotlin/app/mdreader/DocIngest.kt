package app.mdreader

import android.app.Activity
import android.content.ContentResolver
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Build
import android.provider.DocumentsContract
import android.util.Log
import java.io.File
import java.io.FileOutputStream
import java.io.InputStream
import java.security.MessageDigest
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.launch
import org.json.JSONArray
import org.json.JSONObject

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

    /** Request code for ACTION_OPEN_DOCUMENT_TREE. */
    private const val PICK_DIRECTORY_REQUEST = 0x4D54 // "MT" as u16

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
     * Document byte cap (FR-018 / Spec 020) — the Rust constant
     * `player::interruption::MAX_DOC_BYTES`
     * (crates/player/src/interruption.rs), mirrored at the picker
     * boundary. A document past the cap (1 MiB / 1,048,576 bytes) is refused with
     * [REFUSAL_FORMAT_UNDELIVERABLE] before synthesis.
     */
    const val MAX_DOC_BYTES = 1048576

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
     * Launches the system directory tree picker for continuous reading.
     *
     * Releases any previously persisted tree URI grants so that only a
     * single active directory permission is held at any given time.
     * Starts [Intent.ACTION_OPEN_DOCUMENT_TREE] carrying persistable read flags.
     *
     * @param activity The Activity to start the tree picker from (for startActivityForResult).
     * @return true if directory picker was launched, false if the launch failed.
     */
    fun pickDirectory(activity: Activity): Boolean {
        releaseExistingTreePermissions(activity.contentResolver)

        val intent = Intent(Intent.ACTION_OPEN_DOCUMENT_TREE).apply {
            addFlags(
                Intent.FLAG_GRANT_READ_URI_PERMISSION or
                    Intent.FLAG_GRANT_PERSISTABLE_URI_PERMISSION
            )
        }

        return try {
            activity.startActivityForResult(intent, PICK_DIRECTORY_REQUEST)
            true
        } catch (e: Exception) {
            Log.e(TAG, "Failed to start directory picker", e)
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
            // T096: the FR-018 second-document durable save lives in
            // RUST, not here — the outgoing session's last-spoken
            // point is persisted by the cold-start open path (the
            // single playback authority, SC-035) before the outgoing
            // session is torn down and the incoming document takes
            // over. The reflective `saveSessionBookmark` call that
            // used to live here resolved to a method that exists
            // nowhere, so it was a dead no-op; the durable put it
            // claimed to perform is the Rust-side one.

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
            ShellHost.requestFocus()
            val ingestStatus = try {
                onDocumentIngested(
                    result.snapshotPath,
                    result.fingerprintHex,
                    result.originalUri,
                    result.mimeType
                )
            } catch (error: UnsatisfiedLinkError) {
                Log.e(TAG, "Ingest channel unavailable: ${error.message}")
                -1
            }
            
            val outcome = when (ingestStatus) {
                0 -> IngestOutcome.Accepted(result)
                1 -> IngestOutcome.Unreadable
                2 -> IngestOutcome.Refused(REFUSAL_FORMAT_UNDELIVERABLE)
                else -> IngestOutcome.Unreadable
            }
            
            outcome
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
     * Releases existing persisted tree URI permissions to enforce the single-active
     * directory policy (FR-001) and prevent leakage against Android's 512 grant ceiling.
     */
    private fun releaseExistingTreePermissions(resolver: ContentResolver) {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.KITKAT) return

        try {
            val permissions = resolver.persistedUriPermissions
            for (permission in permissions) {
                val uri = permission.uri
                if (DocumentsContract.isTreeUri(uri)) {
                    try {
                        resolver.releasePersistableUriPermission(
                            uri,
                            Intent.FLAG_GRANT_READ_URI_PERMISSION
                        )
                        Log.d(TAG, "Released previous tree URI permission: $uri")
                    } catch (e: SecurityException) {
                        Log.w(TAG, "Failed to release tree permission for $uri", e)
                    }
                }
            }
        } catch (e: Exception) {
            Log.w(TAG, "Failed to inspect persistedUriPermissions for release", e)
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

    /** Background scope for directory scans (Dispatchers.IO, never the UI thread). */
    private val scanScope = CoroutineScope(SupervisorJob() + Dispatchers.IO)

    @Volatile
    private var scanJob: Job? = null

    /** One discovered .md/.txt file; ingest bookkeeping only. */
    private class Found(val name: String, val relativePath: String, val uri: Uri, val extension: String)

    /**
     * Handles the tree-picker result (T198/T199). Takes the persistable
     * grant, then scans on [Dispatchers.IO]: root files first, then each
     * 1st-level subfolder, all in natural order. The first chapter is
     * dispatched to Rust immediately (before the scan completes); the
     * full manifest follows. Kotlin decides nothing about playback.
     *
     * @return true if a scan was launched.
     */
    fun handleDirectoryResult(
        activity: Activity,
        requestCode: Int,
        resultCode: Int,
        data: Intent?
    ): Boolean {
        if (requestCode != PICK_DIRECTORY_REQUEST || resultCode != Activity.RESULT_OK) return false
        val treeUri = data?.data ?: return false
        val context = activity.applicationContext
        val resolver = context.contentResolver
        try {
            resolver.takePersistableUriPermission(treeUri, Intent.FLAG_GRANT_READ_URI_PERMISSION)
        } catch (e: SecurityException) {
            Log.w(TAG, "Persistable tree grant refused; one-shot grant serves the scan", e)
        }
        scanJob?.cancel()
        scanJob = scanScope.launch { scanDirectory(context, resolver, treeUri) }
        return true
    }

    private fun scanDirectory(context: Context, resolver: ContentResolver, treeUri: Uri) {
        try {
            val rootDocId = DocumentsContract.getTreeDocumentId(treeUri)
            val rootName = queryName(resolver, treeUri, rootDocId) ?: rootDocId
            val chapters = JSONArray()
            var firstSent = false

            fun addFile(f: Found) {
                val bytes = try {
                    readDocumentBytes(resolver, f.uri)
                } catch (e: Exception) {
                    Log.w(TAG, "Skipping unreadable chapter ${f.relativePath}", e)
                    return
                }
                if (bytes.size > MAX_DOC_BYTES) {
                    Log.w(TAG, "Skipping chapter past the byte cap: ${f.relativePath}")
                    return
                }
                val hex = bytesToHex(blake3_256(bytes))
                val snapshot = writeSnapshot(context, bytes, f.extension, hex)
                chapters.put(
                    JSONObject()
                        .put("filename", f.name)
                        .put("snapshot_path", snapshot.absolutePath)
                        .put("relative_path", f.relativePath)
                        .put("byte_size", bytes.size)
                        .put("fingerprint_hex", hex)
                )
                if (!firstSent) {
                    firstSent = true
                    // T199: first chapter reaches Rust before the scan completes.
                    dispatchManifest(treeUri, rootName, chapters)
                }
            }

            val rootEntries = listChildren(resolver, treeUri, rootDocId)
            rootEntries.filter { !it.isDir && it.extension != null }
                .sortedWith { a, b -> naturalCompare(a.name, b.name) }
                .forEach { addFile(Found(it.name, it.name, it.uri, it.extension!!)) }
            rootEntries.filter { it.isDir }
                .sortedWith { a, b -> naturalCompare(a.name, b.name) }
                .forEach { dir ->
                    listChildren(resolver, treeUri, dir.docId)
                        .filter { !it.isDir && it.extension != null }
                        .sortedWith { a, b -> naturalCompare(a.name, b.name) }
                        .forEach { addFile(Found(it.name, "${dir.name}/${it.name}", it.uri, it.extension!!)) }
                }
            if (chapters.length() > 0) dispatchManifest(treeUri, rootName, chapters)
            else Log.w(TAG, "Directory holds no .md/.txt chapters: $treeUri")
        } catch (e: Exception) {
            Log.e(TAG, "Directory scan failed", e)
        }
    }

    private fun dispatchManifest(treeUri: Uri, rootName: String, chapters: JSONArray) {
        // Snapshot the array: the scan keeps appending after the first yield.
        val json = JSONObject()
            .put("tree_uri", treeUri.toString())
            .put("root_name", rootName)
            .put("chapters", JSONArray(chapters.toString()))
            .toString()
        try {
            onDirectorySelected(treeUri.toString(), rootName, json)
        } catch (err: UnsatisfiedLinkError) {
            Log.w(TAG, "Native library unavailable; directory manifest not delivered", err)
        }
    }

    private class Entry(val docId: String, val name: String, val uri: Uri, val isDir: Boolean) {
        val extension: String? = name.substringAfterLast('.', "").lowercase().takeIf { it in MIME_BY_EXTENSION }
    }

    private fun queryName(resolver: ContentResolver, treeUri: Uri, docId: String): String? {
        val uri = DocumentsContract.buildDocumentUriUsingTree(treeUri, docId)
        return resolver.query(uri, arrayOf(DocumentsContract.Document.COLUMN_DISPLAY_NAME), null, null, null)
            ?.use { if (it.moveToFirst()) it.getString(0) else null }
    }

    private fun listChildren(resolver: ContentResolver, treeUri: Uri, parentDocId: String): List<Entry> {
        val children = DocumentsContract.buildChildDocumentsUriUsingTree(treeUri, parentDocId)
        val cols = arrayOf(
            DocumentsContract.Document.COLUMN_DOCUMENT_ID,
            DocumentsContract.Document.COLUMN_DISPLAY_NAME,
            DocumentsContract.Document.COLUMN_MIME_TYPE
        )
        val out = ArrayList<Entry>()
        resolver.query(children, cols, null, null, null)?.use { c ->
            while (c.moveToNext()) {
                val id = c.getString(0)
                out.add(
                    Entry(
                        id,
                        c.getString(1) ?: continue,
                        DocumentsContract.buildDocumentUriUsingTree(treeUri, id),
                        c.getString(2) == DocumentsContract.Document.MIME_TYPE_DIR
                    )
                )
            }
        }
        return out
    }

    /** Natural alphanumeric order: digit runs compare numerically, case-insensitive. */
    internal fun naturalCompare(a: String, b: String): Int {
        var i = 0
        var j = 0
        while (i < a.length && j < b.length) {
            if (a[i].isDigit() && b[j].isDigit()) {
                var ie = i
                while (ie < a.length && a[ie].isDigit()) ie++
                var je = j
                while (je < b.length && b[je].isDigit()) je++
                val na = a.substring(i, ie).trimStart('0')
                val nb = b.substring(j, je).trimStart('0')
                if (na.length != nb.length) return na.length - nb.length
                val c = na.compareTo(nb)
                if (c != 0) return c
                i = ie
                j = je
            } else {
                val c = a[i].lowercaseChar().compareTo(b[j].lowercaseChar())
                if (c != 0) return c
                i++
                j++
            }
        }
        return (a.length - i) - (b.length - j)
    }

    /** Native callback: directory manifest (contracts/directory-playback.md §1.3). */
    private external fun onDirectorySelected(treeUri: String, rootName: String, manifestJson: String)

    /**
     * Native callback to Rust after successful document ingestion.
     * Forwards snapshot path, fingerprint, original URI, and MIME type.
     */
    @JvmStatic
    private external fun onDocumentIngested(
        snapshotPath: String,
        fingerprintHex: String,
        originalUri: String,
        mimeType: String
    ): Int

    // --- Debug/test automation hooks (BuildConfig.DEBUG-gated callers only) ---
    internal fun debugDeliverDocument(
        context: Context,
        bytes: ByteArray,
        extension: String,
        originalUri: String,
        mimeType: String,
        sourceName: String
    ) {
        val hex = bytesToHex(blake3_256(bytes))
        val snapshot = writeSnapshot(context, bytes, extension, hex)
        currentSnapshot = snapshot
        trimSnapshots(context, snapshot)
        val status = onDocumentIngested(snapshot.absolutePath, hex, originalUri, mimeType)
        Log.i(TAG, "debugDeliverDocument: $sourceName -> ${snapshot.absolutePath} (jniStatus=$status)")
    }

    /**
     * Spec 019: Ingest incoming Intent from ACTION_VIEW (open .md file) or ACTION_SEND (share sheet).
     *
     * Extracts URI or text stream, creates durable snapshot and delegates to onDocumentIngested JNI.
     */
    fun handleIncomingIntent(activity: Activity, intent: Intent?): Boolean {
        if (intent == null) return false
        val action = intent.action ?: return false
        val resolver = activity.contentResolver

        Log.i(TAG, "handleIncomingIntent: action=$action type=${intent.type} data=${intent.data}")

        val (bytes, uriStr, extension) = when (action) {
            Intent.ACTION_VIEW -> {
                val uri = intent.data ?: return false
                val ext = resolveFormat(resolver, uri) ?: "md"
                val rawBytes = try {
                    readDocumentBytes(resolver, uri)
                } catch (e: Exception) {
                    Log.e(TAG, "handleIncomingIntent: failed to read ACTION_VIEW uri=$uri", e)
                    return false
                }
                Triple(rawBytes, uri.toString(), ext)
            }
            Intent.ACTION_SEND -> {
                val streamUri = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                    intent.getParcelableExtra(Intent.EXTRA_STREAM, Uri::class.java)
                } else {
                    @Suppress("DEPRECATION")
                    intent.getParcelableExtra(Intent.EXTRA_STREAM) as? Uri
                }

                if (streamUri != null) {
                    val ext = resolveFormat(resolver, streamUri) ?: "md"
                    val rawBytes = try {
                        readDocumentBytes(resolver, streamUri)
                    } catch (e: Exception) {
                        Log.e(TAG, "handleIncomingIntent: failed to read ACTION_SEND streamUri=$streamUri", e)
                        return false
                    }
                    Triple(rawBytes, streamUri.toString(), ext)
                } else {
                    val text = intent.getStringExtra(Intent.EXTRA_TEXT) ?: return false
                    val textBytes = text.toByteArray(Charsets.UTF_8)
                    Triple(textBytes, "content://shared-text/${System.currentTimeMillis()}.md", "md")
                }
            }
            else -> return false
        }

        if (bytes.isEmpty() || bytes.size > MAX_DOC_BYTES) {
            Log.w(TAG, "handleIncomingIntent: document empty or exceeds MAX_DOC_BYTES (${bytes.size} bytes)")
            return false
        }

        return try {
            val fingerprint = blake3_256(bytes)
            val hexName = bytesToHex(fingerprint)
            val snapshotFile = writeSnapshot(activity, bytes, extension, hexName)
            currentSnapshot = snapshotFile
            trimSnapshots(activity, snapshotFile)

            val mime = MIME_BY_EXTENSION[extension] ?: "text/markdown"
            ShellHost.requestFocus()
            val status = onDocumentIngested(snapshotFile.absolutePath, hexName, uriStr, mime)
            Log.i(TAG, "handleIncomingIntent: successfully ingested snapshot=${snapshotFile.absolutePath} (jniStatus=$status)")
            status == 0
        } catch (e: Exception) {
            Log.e(TAG, "handleIncomingIntent: snapshot delivery failed", e)
            false
        }
    }

    internal fun debugFingerprintHex(bytes: ByteArray): String = bytesToHex(blake3_256(bytes))
    internal fun debugWriteSnapshot(context: Context, bytes: ByteArray, extension: String, fingerprintHex: String): File =
        writeSnapshot(context, bytes, extension, fingerprintHex)
    internal fun debugDispatchDirectory(treeUri: String, rootName: String, chapters: JSONArray) {
        dispatchManifest(Uri.parse(treeUri), rootName, chapters)
    }

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
