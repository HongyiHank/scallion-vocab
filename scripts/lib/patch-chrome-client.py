#!/usr/bin/env python3
"""
patch-chrome-client.py
Patch RustWebChromeClient.kt: ensure required imports and onCreateWindow method.

Usage:
    python3 scripts/patch-chrome-client.py <path-to-RustWebChromeClient.kt>
"""

import sys

path = sys.argv[1]
with open(path, 'r') as f:
    content = f.read()

needed_imports = [
    ('import android.os.Message', 'import android.os.Build'),
    ('import android.view.ViewGroup', 'import android.view.View'),
    ('import android.graphics.Bitmap', 'import android.view.View'),
    ('import android.widget.Toast', 'import android.widget.EditText'),
]

for imp, after in needed_imports:
    if imp not in content:
        content = content.replace(after, after + '\n' + imp, 1)

if 'private fun openExternalBrowser' not in content:
    method = '''
  override fun onCreateWindow(
    view: WebView?,
    isDialog: Boolean,
    isUserGesture: Boolean,
    resultMsg: Message?
  ): Boolean {
    if (view == null || resultMsg == null) return false
    val context = view.context
    val popup = WebView(context)
    var consumed = false

    fun consumePopupUrl(url: String): Boolean {
      if (consumed) return true
      consumed = true
      popup.stopLoading()
      try {
        (popup.parent as? ViewGroup)?.removeView(popup)
      } catch (_: Throwable) { }
      popup.destroy()
      openExternalBrowser(context, url)
      return true
    }

    popup.settings.apply {
      javaScriptEnabled = true
      domStorageEnabled = true
      javaScriptCanOpenWindowsAutomatically = true
      setSupportMultipleWindows(false)
      if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.N) {
        setDisabledActionModeMenuItems(WebSettings.MENU_ITEM_NONE)
      }
    }

    popup.webViewClient = object : WebViewClient() {
      override fun shouldOverrideUrlLoading(
        v: WebView, request: WebResourceRequest
      ): Boolean {
        return consumePopupUrl(request.url.toString())
      }

      @Suppress("DEPRECATION")
      override fun shouldOverrideUrlLoading(v: WebView, url: String): Boolean {
        return consumePopupUrl(url)
      }

      override fun onPageStarted(v: WebView, url: String, favicon: Bitmap?) {
        consumePopupUrl(url)
      }
    }

    // Attach popup to parent (some WebView builds need it attached)
    try {
      (view.parent as? ViewGroup)?.addView(
        popup, ViewGroup.LayoutParams(1, 1)
      )
    } catch (_: Throwable) { }

    val transport = resultMsg.obj as? WebView.WebViewTransport
    transport?.webView = popup
    resultMsg.sendToTarget()
    return true
  }

  private fun openExternalBrowser(context: android.content.Context, rawUrl: String) {
    val uri = try {
      android.net.Uri.parse(rawUrl)
    } catch (_: Throwable) {
      Toast.makeText(context, "URL \u683C\u5F0F\u932F\u8AA4", Toast.LENGTH_SHORT).show()
      return
    }
    val scheme = uri.scheme?.lowercase()
    if (scheme != "http" && scheme != "https") {
      Toast.makeText(context, "\u53EA\u5141\u8A31\u958B\u555F http/https \u9023\u7D50", Toast.LENGTH_SHORT).show()
      return
    }
    try {
      context.startActivity(Intent(Intent.ACTION_VIEW, uri).apply {
        addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
      })
    } catch (_: ActivityNotFoundException) {
      Toast.makeText(context, "\u627E\u4E0D\u5230\u53EF\u958B\u555F\u9023\u7D50\u7684\u700F\u89BD\u5668", Toast.LENGTH_SHORT).show()
    }
  }

'''
    # Insert before `init {`
    content = content.replace('  init {', method + '  init {', 1)

with open(path, 'w') as f:
    f.write(content)
