package org.fleetranger.rc

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent

/**
 * Starts uploading again after the controller boots, and after an update of
 * the app; then also says so, as Android closed the app to replace it.
 */
class BootReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        when (intent.action) {
            Intent.ACTION_BOOT_COMPLETED -> UploadService.start(context)
            Intent.ACTION_MY_PACKAGE_REPLACED -> {
                UploadService.start(context)
                Updates.notifyInstalled(context)
            }
        }
    }
}
