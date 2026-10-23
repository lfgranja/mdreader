package com.mdreader.focusinjector

import android.annotation.SuppressLint
import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.media.AudioAttributes
import android.media.AudioFocusRequest
import android.media.AudioManager
import android.os.Build
import android.os.Bundle
import android.util.Log
import android.widget.Button
import android.widget.TextView
import android.widget.Toast
import androidx.appcompat.app.AppCompatActivity
import androidx.core.app.NotificationCompat
import androidx.core.content.ContextCompat
import com.mdreader.focusinjector.R

class MainActivity : AppCompatActivity() {

    private lateinit var audioManager: AudioManager
    private var focusRequest: AudioFocusRequest? = null
    private var currentFocusType: String = ""
    private var isFocusHeld = false
    private lateinit var statusText: TextView
    private lateinit var btnDuckable: Button
    private lateinit var btnSilence: Button
    private lateinit var btnPermanent: Button
    private lateinit var btnAbandon: Button
    private lateinit var btnBecomingNoisy: Button
    private lateinit var btnDeviceRemoved: Button

    private val TAG = "FocusInjector"

    companion object {
        const val CHANNEL_ID = "focus_injector_channel"
        const val NOTIFICATION_ID = 1
        const val FOCUS_DURATION_MS = 30_000 // 30 seconds default hold
    }

    @SuppressLint("MissingPermission")
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        audioManager = getSystemService(Context.AUDIO_SERVICE) as AudioManager

        statusText = findViewById(R.id.statusText)
        btnDuckable = findViewById(R.id.btnDuckable)
        btnSilence = findViewById(R.id.btnSilence)
        btnPermanent = findViewById(R.id.btnPermanent)
        btnAbandon = findViewById(R.id.btnAbandon)
        btnBecomingNoisy = findViewById(R.id.btnBecomingNoisy)
        btnDeviceRemoved = findViewById(R.id.btnDeviceRemoved)

        createNotificationChannel()

        btnDuckable.setOnClickListener { requestFocus(AudioManager.AUDIOFOCUS_GAIN_TRANSIENT_MAY_DUCK, "DuckableTransient (MAY_DUCK)") }
        btnSilence.setOnClickListener { requestFocus(AudioManager.AUDIOFOCUS_GAIN_TRANSIENT, "SilenceDemanding (TRANSIENT)") }
        btnPermanent.setOnClickListener { requestFocus(AudioManager.AUDIOFOCUS_GAIN, "PermanentLoss (GAIN)") }
        btnAbandon.setOnClickListener { abandonFocus() }
        btnBecomingNoisy.setOnClickListener { sendBecomingNoisy() }
        btnDeviceRemoved.setOnClickListener { sendDeviceRemoved() }

        updateUI()
    }

    private fun createNotificationChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = NotificationChannel(
                CHANNEL_ID,
                "Focus Injector",
                NotificationManager.IMPORTANCE_LOW
            )
            channel.description = "Keeps focus injector alive while holding audio focus"
            val manager = getSystemService(NotificationManager::class.java)
            manager.createNotificationChannel(channel)
        }
    }

    @SuppressLint("MissingPermission")
    private fun requestFocus(focusGain: Int, label: String) {
        if (isFocusHeld) {
            Toast.makeText(this, "Already holding focus: $currentFocusType. Press ABANDON first.", Toast.LENGTH_SHORT).show()
            return
        }

        val attributes = AudioAttributes.Builder()
            .setUsage(AudioAttributes.USAGE_MEDIA)
            .setContentType(AudioAttributes.CONTENT_TYPE_SPEECH)
            .build()

        val builder = AudioFocusRequest.Builder(focusGain)
            .setAudioAttributes(attributes)
            .setWillPauseWhenDucked(true)
            .setAcceptsDelayedFocusGain(true)
            .setOnAudioFocusChangeListener { focusChange ->
                Log.d(TAG, "Focus change: $focusChange (${focusChangeToString(focusChange)})")
                runOnUiThread {
                    when (focusChange) {
                        AudioManager.AUDIOFOCUS_LOSS -> {
                            // Permanent loss - we lost focus
                            Toast.makeText(this@MainActivity, "AUDIOFOCUS_LOSS - focus lost permanently", Toast.LENGTH_LONG).show()
                        }
                        AudioManager.AUDIOFOCUS_LOSS_TRANSIENT -> {
                            Toast.makeText(this@MainActivity, "AUDIOFOCUS_LOSS_TRANSIENT - temporary loss", Toast.LENGTH_SHORT).show()
                        }
                        AudioManager.AUDIOFOCUS_LOSS_TRANSIENT_CAN_DUCK -> {
                            Toast.makeText(this@MainActivity, "AUDIOFOCUS_LOSS_TRANSIENT_CAN_DUCK - can duck", Toast.LENGTH_SHORT).show()
                        }
                        AudioManager.AUDIOFOCUS_GAIN -> {
                            Toast.makeText(this@MainActivity, "AUDIOFOCUS_GAIN - focus regained", Toast.LENGTH_SHORT).show()
                        }
                    }
                }
            }

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            focusRequest = builder.build()
            val result = audioManager.requestAudioFocus(focusRequest!!)
            handleFocusResult(result, label)
        } else {
            // Pre-Oreo fallback
            val result = audioManager.requestAudioFocus({ focusChange ->
                Log.d(TAG, "Focus change (legacy): $focusChange")
            }, AudioManager.STREAM_MUSIC, focusGain)
            handleFocusResult(result, label)
        }
    }

    private fun handleFocusResult(result: Int, label: String) {
        when (result) {
            AudioManager.AUDIOFOCUS_REQUEST_GRANTED -> {
                isFocusHeld = true
                currentFocusType = label
                Toast.makeText(this, getString(R.string.toast_focus_requested, label), Toast.LENGTH_SHORT).show()
                Log.d(TAG, "Focus GRANTED: $label")
                startForegroundService()
            }
            AudioManager.AUDIOFOCUS_REQUEST_DELAYED -> {
                Toast.makeText(this, "Focus DELAYED: $label (will callback when granted)", Toast.LENGTH_LONG).show()
                Log.d(TAG, "Focus DELAYED: $label")
            }
            AudioManager.AUDIOFOCUS_REQUEST_FAILED -> {
                Toast.makeText(this, getString(R.string.toast_error, "Focus request FAILED for $label"), Toast.LENGTH_LONG).show()
                Log.e(TAG, "Focus FAILED: $label")
            }
            else -> {
                Toast.makeText(this, getString(R.string.toast_error, "Unknown result: $result"), Toast.LENGTH_LONG).show()
            }
        }
        updateUI()
    }

    @SuppressLint("MissingPermission")
    private fun abandonFocus() {
        if (!isFocusHeld) {
            Toast.makeText(this, "No focus currently held", Toast.LENGTH_SHORT).show()
            return
        }

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O && focusRequest != null) {
            audioManager.abandonAudioFocusRequest(focusRequest!!)
        } else {
            audioManager.abandonAudioFocus(null)
        }

        isFocusHeld = false
        currentFocusType = ""
        focusRequest = null
        stopForegroundService()
        Toast.makeText(this, R.string.toast_focus_abandoned, Toast.LENGTH_SHORT).show()
        Log.d(TAG, "Focus abandoned")
        updateUI()
    }

    private fun sendBecomingNoisy() {
        // Send ACTION_AUDIO_BECOMING_NOISY broadcast - this simulates headphone unplug
        val intent = Intent(AudioManager.ACTION_AUDIO_BECOMING_NOISY)
        intent.addFlags(Intent.FLAG_RECEIVER_REGISTERED_ONLY)
        sendBroadcast(intent)
        Toast.makeText(this, "Sent ACTION_AUDIO_BECOMING_NOISY broadcast", Toast.LENGTH_SHORT).show()
        Log.d(TAG, "Sent ACTION_AUDIO_BECOMING_NOISY")
    }

    private fun sendDeviceRemoved() {
        // Note: We can't directly call onAudioDevicesRemoved - that's a system callback
        // But we can document that this requires a real device disconnect
        Toast.makeText(this, "Device removal must be physical (unplug headphones/turn off BT). Use real device for this trial.", Toast.LENGTH_LONG).show()
        Log.d(TAG, "Device removal trial - requires physical action")
    }

    @SuppressLint("MissingPermission")
    private fun startForegroundService() {
        val intent = Intent(this, MainActivity::class.java)
        val pendingIntent = PendingIntent.getActivity(
            this, 0, intent,
            PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT
        )

        val notification = NotificationCompat.Builder(this, CHANNEL_ID)
            .setContentTitle("Focus Injector")
            .setContentText("Holding focus: $currentFocusType")
            .setSmallIcon(android.R.drawable.ic_media_play)
            .setContentIntent(pendingIntent)
            .setOngoing(true)
            .setPriority(NotificationCompat.PRIORITY_LOW)
            .build()

        // Show notification to keep app visible while holding focus
        val manager = getSystemService(NotificationManager::class.java)
        manager.notify(NOTIFICATION_ID, notification)
    }

    private fun stopForegroundService() {
        val manager = getSystemService(NotificationManager::class.java)
        manager.cancel(NOTIFICATION_ID)
    }

    private fun updateUI() {
        if (isFocusHeld) {
            statusText.text = getString(R.string.status_active, currentFocusType)
            btnAbandon.isEnabled = true
            btnDuckable.isEnabled = false
            btnSilence.isEnabled = false
            btnPermanent.isEnabled = false
        } else {
            statusText.text = getString(R.string.status_ready)
            btnAbandon.isEnabled = false
            btnDuckable.isEnabled = true
            btnSilence.isEnabled = true
            btnPermanent.isEnabled = true
        }
    }

    private fun focusChangeToString(change: Int): String {
        return when (change) {
            AudioManager.AUDIOFOCUS_GAIN -> "GAIN"
            AudioManager.AUDIOFOCUS_LOSS -> "LOSS"
            AudioManager.AUDIOFOCUS_LOSS_TRANSIENT -> "LOSS_TRANSIENT"
            AudioManager.AUDIOFOCUS_LOSS_TRANSIENT_CAN_DUCK -> "LOSS_TRANSIENT_CAN_DUCK"
            AudioManager.AUDIOFOCUS_REQUEST_GRANTED -> "REQUEST_GRANTED"
            AudioManager.AUDIOFOCUS_REQUEST_DELAYED -> "REQUEST_DELAYED"
            AudioManager.AUDIOFOCUS_REQUEST_FAILED -> "REQUEST_FAILED"
            else -> "UNKNOWN($change)"
        }
    }

    override fun onDestroy() {
        super.onDestroy()
        if (isFocusHeld) {
            abandonFocus()
        }
    }
}