package dev.dioxus.main

import android.app.Activity
import android.content.ActivityNotFoundException
import android.content.Intent
import android.net.Uri
import android.os.Bundle
import android.provider.DocumentsContract
import android.provider.OpenableColumns
import android.util.Base64
import android.webkit.JavascriptInterface
import android.webkit.WebView
import androidx.activity.result.contract.ActivityResultContracts
import androidx.annotation.RequiresApi
import org.json.JSONObject
import java.nio.charset.StandardCharsets

class MainActivity : WryActivity() {
    private var appWebView: WebView? = null
    private var pendingPickerRequestId: String? = null

    private val pickerLauncher = registerForActivityResult(
        ActivityResultContracts.StartActivityForResult()
    ) { result ->
        val requestId = pendingPickerRequestId ?: return@registerForActivityResult
        pendingPickerRequestId = null
        if (result.resultCode != Activity.RESULT_OK || result.data == null) {
            respond(requestId, resultObject("cancelled"))
            return@registerForActivityResult
        }

        val data = result.data ?: run {
            respond(requestId, errorObject("attachment_unavailable", "The provider returned no file"))
            return@registerForActivityResult
        }
        val uri = data.data ?: run {
            respond(requestId, errorObject("attachment_unavailable", "The provider returned no file"))
            return@registerForActivityResult
        }
        val grantedRead = data.flags and Intent.FLAG_GRANT_READ_URI_PERMISSION
        if (grantedRead == 0) {
            respond(requestId, errorObject("permission_denied", "The provider did not grant read access"))
            return@registerForActivityResult
        }
        try {
            contentResolver.takePersistableUriPermission(uri, grantedRead)
        } catch (_: SecurityException) {
            respond(requestId, errorObject("permission_denied", "The provider did not grant persistent read access"))
            return@registerForActivityResult
        }

        val displayName = queryDisplayName(uri)
        val mimeType = contentResolver.getType(uri)
        val resultObject = resultObject("document_selected")
            .put("uri", uri.toString())
            .put("display_name", displayName ?: JSONObject.NULL)
            .put("mime_type", mimeType ?: JSONObject.NULL)
        respond(requestId, resultObject)
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        pendingPickerRequestId = savedInstanceState?.getString(PENDING_PICKER_KEY)
    }

    override fun onSaveInstanceState(outState: Bundle) {
        outState.putString(PENDING_PICKER_KEY, pendingPickerRequestId)
        super.onSaveInstanceState(outState)
    }

    override fun onWebViewCreate(webView: WebView) {
        super.onWebViewCreate(webView)
        appWebView = webView
        webView.settings.javaScriptEnabled = true
        webView.addJavascriptInterface(NativeBridge(), BRIDGE_NAME)
    }

    private inner class NativeBridge {
        @JavascriptInterface
        fun postMessageBase64(payload: String) {
            runOnUiThread {
                if (!isTrustedPackagedPage()) {
                    respond("unknown", errorObject("bridge_unavailable", "Untrusted page"))
                    return@runOnUiThread
                }
                handlePayload(payload)
            }
        }
    }

    private fun handlePayload(payload: String) {
        val request = try {
            val bytes = Base64.decode(payload, Base64.DEFAULT)
            JSONObject(String(bytes, StandardCharsets.UTF_8))
        } catch (_: IllegalArgumentException) {
            respond("unknown", errorObject("malformed_request", "Malformed request"))
            return
        }

        val version = request.optInt("version", -1)
        val requestId = request.optString("request_id", "")
        if (version != PROTOCOL_VERSION || requestId.isBlank()) {
            respond(requestId.ifBlank { "unknown" }, errorObject("protocol_version", "Unsupported request"))
            return
        }
        val operation = request.optJSONObject("operation")
        val kind = operation?.optString("kind", "") ?: ""
        when (kind) {
            "app_data_directory" -> {
                respond(requestId, resultObject("app_data_directory").put("path", filesDir.absolutePath))
            }
            "pick_document" -> launchPicker(requestId, operation.optBoolean("prefer_downloads", true))
            "open_document" -> openDocument(requestId, operation)
            "open_url" -> openUrl(requestId, operation)
            "release_read_permission" -> releaseReadPermission(requestId, operation)
            else -> respond(requestId, errorObject("unknown_operation", "Unsupported operation"))
        }
    }

    private fun launchPicker(requestId: String, preferDownloads: Boolean) {
        if (pendingPickerRequestId != null) {
            respond(requestId, errorObject("picker_busy", "Another file picker is already open"))
            return
        }
        pendingPickerRequestId = requestId
        val intent = Intent(Intent.ACTION_OPEN_DOCUMENT).apply {
            addCategory(Intent.CATEGORY_OPENABLE)
            type = "*/*"
            addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
            addFlags(Intent.FLAG_GRANT_PERSISTABLE_URI_PERMISSION)
            if (preferDownloads) {
                putExtra(DocumentsContract.EXTRA_INITIAL_URI, downloadsRootUri())
            }
        }
        try {
            pickerLauncher.launch(intent)
        } catch (_: ActivityNotFoundException) {
            pendingPickerRequestId = null
            respond(requestId, errorObject("no_handler", "No document provider is installed"))
        } catch (_: RuntimeException) {
            pendingPickerRequestId = null
            respond(requestId, errorObject("picker_unavailable", "The document picker could not be opened"))
        }
    }

    private fun openDocument(requestId: String, operation: JSONObject) {
        val uriText = operation.optString("uri", "")
        if (uriText.isBlank()) {
            respond(requestId, errorObject("attachment_unavailable", "The attachment URI is empty"))
            return
        }
        val mimeType = operation.optString("mime_type", "").ifBlank { "*/*" }
        try {
            val intent = Intent(Intent.ACTION_VIEW).apply {
                setDataAndType(Uri.parse(uriText), mimeType)
                addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
            }
            startActivity(intent)
            respond(requestId, resultObject("completed"))
        } catch (_: ActivityNotFoundException) {
            respond(requestId, errorObject("no_handler", "No installed app can open this file"))
        } catch (_: SecurityException) {
            respond(requestId, errorObject("attachment_unavailable", "The attachment is no longer accessible"))
        } catch (_: IllegalArgumentException) {
            respond(requestId, errorObject("attachment_unavailable", "The attachment URI is invalid"))
        }
    }

    private fun openUrl(requestId: String, operation: JSONObject) {
        val urlText = operation.optString("url", "")
        val uri = try {
            Uri.parse(urlText)
        } catch (_: IllegalArgumentException) {
            null
        }
        if (uri == null || (uri.scheme != "http" && uri.scheme != "https")) {
            respond(requestId, errorObject("unsupported_url", "Only HTTP and HTTPS links are supported"))
            return
        }
        try {
            startActivity(Intent(Intent.ACTION_VIEW, uri))
            respond(requestId, resultObject("completed"))
        } catch (_: ActivityNotFoundException) {
            respond(requestId, errorObject("no_handler", "No browser is installed"))
        }
    }

    private fun releaseReadPermission(requestId: String, operation: JSONObject) {
        val uriText = operation.optString("uri", "")
        if (uriText.isNotBlank()) {
            try {
                contentResolver.releasePersistableUriPermission(
                    Uri.parse(uriText),
                    Intent.FLAG_GRANT_READ_URI_PERMISSION
                )
            } catch (_: SecurityException) {
                // Releasing an already revoked grant is intentionally idempotent.
            } catch (_: IllegalArgumentException) {
                // The database may contain a provider URI that has already disappeared.
            }
        }
        respond(requestId, resultObject("completed"))
    }

    private fun respond(requestId: String, result: JSONObject) {
        val response = JSONObject()
            .put("version", PROTOCOL_VERSION)
            .put("request_id", requestId)
            .put("result", result)
        val encoded = Base64.encodeToString(response.toString().toByteArray(StandardCharsets.UTF_8), Base64.NO_WRAP)
        val script = "window.__chinaTravelNativeResolveBase64 && window.__chinaTravelNativeResolveBase64('$encoded');"
        appWebView?.post { appWebView?.evaluateJavascript(script, null) }
    }

    private fun resultObject(kind: String): JSONObject = JSONObject().put("kind", kind)

    private fun errorObject(code: String, message: String): JSONObject = resultObject("error")
        .put("code", code)
        .put("message", message)

    private fun queryDisplayName(uri: Uri): String? {
        return contentResolver.query(uri, arrayOf(OpenableColumns.DISPLAY_NAME), null, null, null)?.use { cursor ->
            if (cursor.moveToFirst()) cursor.getString(0) else null
        }
    }

    private fun downloadsRootUri(): Uri = Uri.parse(
        "content://com.android.providers.downloads.documents/root/downloads"
    )

    private fun isTrustedPackagedPage(): Boolean {
        val url = appWebView?.url ?: return true
        return url.startsWith("http://dioxus.") ||
            url.startsWith("https://dioxus.") ||
            url.startsWith("file:///android_asset/")
    }

    companion object {
        private const val BRIDGE_NAME = "ChinaTravelBridge"
        private const val PENDING_PICKER_KEY = "chinaTravel.pendingPickerRequestId"
        private const val PROTOCOL_VERSION = 1
    }
}

internal object BuildConfig {
    val DEBUG: Boolean = com.rdelacruz.chinatravel.BuildConfig.DEBUG
}
