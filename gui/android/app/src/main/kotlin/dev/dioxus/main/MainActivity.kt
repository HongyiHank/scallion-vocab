package dev.dioxus.main

import android.app.Activity
import android.content.ActivityNotFoundException
import android.content.Intent
import android.net.Uri
import android.os.Handler
import android.os.Looper
import android.view.View
import android.view.ViewGroup
import android.webkit.JavascriptInterface
import android.webkit.WebResourceRequest
import android.webkit.WebResourceResponse
import android.webkit.WebView
import android.webkit.WebViewClient
import android.widget.FrameLayout
import android.widget.Toast
import java.io.ByteArrayInputStream

typealias BuildConfig = com.scallion.vocab.BuildConfig

class MainActivity : WryActivity() {

    private lateinit var mainWebView: WebView

    override fun onWebViewCreate(webView: WebView) {
        mainWebView = webView
        registerExternalUrlBridge(webView)
        webView.settings.apply {
            javaScriptEnabled = true
            domStorageEnabled = true
        }
    }

    private fun registerExternalUrlBridge(webView: WebView) {
        webView.addJavascriptInterface(ExternalOpener(this), "AndroidExternal")
        webView.addJavascriptInterface(QuizletFetcher(this), "AndroidQuizletFetcher")
    }

    class ExternalOpener(private val activity: Activity) {
        @JavascriptInterface
        fun openUrl(rawUrl: String) {
            activity.runOnUiThread {
                val uri = Uri.parse(rawUrl)
                val scheme = uri.scheme?.lowercase()

                if (scheme != "http" && scheme != "https") {
                    Toast.makeText(activity, "只允許 http/https 連結", Toast.LENGTH_SHORT).show()
                    return@runOnUiThread
                }

                try {
                    activity.startActivity(Intent(Intent.ACTION_VIEW, uri))
                } catch (_: ActivityNotFoundException) {
                    Toast.makeText(activity, "找不到可開啟連結的瀏覽器", Toast.LENGTH_SHORT).show()
                }
            }
        }
    }

    class QuizletFetcher(private val activity: MainActivity) {
        @Volatile
        private var resultSent = false
        private var hiddenWebView: WebView? = null
        private val handler = Handler(Looper.getMainLooper())

        @JavascriptInterface
        fun fetchQuizlet(url: String) {
            activity.runOnUiThread {
                cleanup()
                resultSent = false

                val hidden = WebView(activity)
                hidden.settings.javaScriptEnabled = true
                hidden.settings.domStorageEnabled = true
                hidden.settings.userAgentString = null
                hidden.settings.blockNetworkImage = true
                hidden.visibility = View.INVISIBLE

                hidden.webViewClient = object : WebViewClient() {
                    override fun shouldInterceptRequest(
                        view: WebView,
                        request: WebResourceRequest
                    ): WebResourceResponse? {
                        if (request.isForMainFrame) return null
                        val path = request.url.path?.lowercase() ?: return null
                        if (path.endsWith(".woff") || path.endsWith(".woff2") ||
                            path.endsWith(".ttf") || path.endsWith(".eot") ||
                            path.endsWith(".svg") || path.endsWith(".ico")
                        ) {
                            return WebResourceResponse(
                                "text/plain", "utf-8",
                                ByteArrayInputStream(ByteArray(0))
                            )
                        }
                        return null
                    }

                    override fun onPageFinished(view: WebView, pageUrl: String) {
                        if (resultSent) return
                        pollForContent(view, 0)
                    }
                }

                val decorView = activity.window.decorView as ViewGroup
                decorView.addView(
                    hidden,
                    FrameLayout.LayoutParams(
                        FrameLayout.LayoutParams.MATCH_PARENT,
                        FrameLayout.LayoutParams.MATCH_PARENT
                    )
                )
                hiddenWebView = hidden

                hidden.loadUrl(url)

                handler.postDelayed({
                    if (!resultSent) sendResult("''")
                }, 15000)
            }
        }

        private fun pollForContent(view: WebView, attempt: Int) {
            if (resultSent || attempt >= 40) return

            view.evaluateJavascript("document.documentElement.outerHTML") { result ->
                if (resultSent) return@evaluateJavascript

                val htmlJs = result ?: "null"
                if (looksLikeRealContent(htmlJs)) {
                    sendResult(htmlJs)
                } else {
                    handler.postDelayed({ pollForContent(view, attempt + 1) }, 300)
                }
            }
        }

        private fun looksLikeRealContent(htmlJs: String): Boolean {
            return htmlJs.contains("__NEXT_DATA__") ||
                htmlJs.contains("OrderedDictionary")
        }

        private fun sendResult(htmlJs: String) {
            if (resultSent) return
            resultSent = true
            handler.removeCallbacksAndMessages(null)

            activity.mainWebView.evaluateJavascript(
                "window.__quizletFetchComplete && window.__quizletFetchComplete($htmlJs)"
            ) {}

            cleanup()
        }

        private fun cleanup() {
            hiddenWebView?.let {
                (it.parent as? ViewGroup)?.removeView(it)
                it.destroy()
            }
            hiddenWebView = null
        }
    }
}
