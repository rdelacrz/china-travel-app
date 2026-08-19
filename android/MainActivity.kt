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
import org.json.JSONException
import org.json.JSONObject
import java.nio.charset.StandardCharsets

class MainActivity : WryActivity() {
    private var appWebView: WebView? = null
    private var pendingPickerRequestId: String? = null
    private var pendingCreateRequestId: String? = null
    private var pendingCreateContent: ByteArray? = null

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

    private val createLauncher = registerForActivityResult(
        ActivityResultContracts.StartActivityForResult()
    ) { result ->
        val requestId = pendingCreateRequestId ?: return@registerForActivityResult
        val content = pendingCreateContent
        pendingCreateRequestId = null
        pendingCreateContent = null
        if (result.resultCode != Activity.RESULT_OK || result.data?.data == null) {
            respond(requestId, resultObject("cancelled"))
            return@registerForActivityResult
        }
        try {
            contentResolver.openOutputStream(result.data!!.data!!, "wt")?.use { output ->
                output.write(content ?: ByteArray(0))
            } ?: throw IllegalStateException("Provider returned no output stream")
            respond(requestId, resultObject("completed"))
        } catch (_: Exception) {
            respond(requestId, errorObject("write_failed", "The backup file could not be written"))
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        pendingPickerRequestId = savedInstanceState?.getString(PENDING_PICKER_KEY)
        pendingCreateRequestId = savedInstanceState?.getString(PENDING_CREATE_KEY)
    }

    override fun onSaveInstanceState(outState: Bundle) {
        outState.putString(PENDING_PICKER_KEY, pendingPickerRequestId)
        outState.putString(PENDING_CREATE_KEY, pendingCreateRequestId)
        super.onSaveInstanceState(outState)
    }

    @Suppress("DEPRECATION")
    override fun onBackPressed() {
        val webView = appWebView
        if (webView?.canGoBack() == true) {
            webView.goBack()
        } else {
            super.onBackPressed()
        }
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
                    respond(requestIdFromPayload(payload), errorObject("bridge_unavailable", "Untrusted page"))
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
        } catch (_: JSONException) {
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
            "create_document" -> createDocument(requestId, operation)
            "read_text_document" -> readTextDocument(requestId, operation)
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

    private fun createDocument(requestId: String, operation: JSONObject) {
        if (pendingCreateRequestId != null) {
            respond(requestId, errorObject("picker_busy", "Another file picker is already open"))
            return
        }
        val content = try {
            Base64.decode(operation.optString("content_base64", ""), Base64.DEFAULT)
        } catch (_: IllegalArgumentException) {
            respond(requestId, errorObject("malformed_request", "Backup content was invalid"))
            return
        }
        pendingCreateRequestId = requestId
        pendingCreateContent = content
        val intent = Intent(Intent.ACTION_CREATE_DOCUMENT).apply {
            addCategory(Intent.CATEGORY_OPENABLE)
            type = operation.optString("mime_type", "application/json")
            putExtra(Intent.EXTRA_TITLE, operation.optString("file_name", "china_travel_app_backup.json"))
            putExtra(DocumentsContract.EXTRA_INITIAL_URI, downloadsRootUri())
        }
        try {
            createLauncher.launch(intent)
        } catch (_: RuntimeException) {
            pendingCreateRequestId = null
            pendingCreateContent = null
            respond(requestId, errorObject("picker_unavailable", "The save-file picker could not be opened"))
        }
    }

    private fun readTextDocument(requestId: String, operation: JSONObject) {
        val uri = try { Uri.parse(operation.optString("uri", "")) } catch (_: Exception) { null }
        if (uri == null) {
            respond(requestId, errorObject("attachment_unavailable", "The backup URI is invalid"))
            return
        }
        try {
            val content = contentResolver.openInputStream(uri)
                ?.bufferedReader(StandardCharsets.UTF_8)
                ?.use { it.readText() }
                ?: throw IllegalStateException("Provider returned no input stream")
            respond(requestId, resultObject("text_document").put("content", content))
        } catch (_: Exception) {
            respond(requestId, errorObject("attachment_unavailable", "The backup file could not be read"))
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

    private fun requestIdFromPayload(payload: String): String {
        return try {
            val bytes = Base64.decode(payload, Base64.DEFAULT)
            JSONObject(String(bytes, StandardCharsets.UTF_8)).optString("request_id", "unknown")
        } catch (_: IllegalArgumentException) {
            "unknown"
        } catch (_: JSONException) {
            "unknown"
        }
    }

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
        private const val PENDING_CREATE_KEY = "chinaTravel.pendingCreateRequestId"
        private const val PROTOCOL_VERSION = 1
    }
}

internal object BuildConfig {
    val DEBUG: Boolean = com.rdelacruz.chinatravel.BuildConfig.DEBUG
}
