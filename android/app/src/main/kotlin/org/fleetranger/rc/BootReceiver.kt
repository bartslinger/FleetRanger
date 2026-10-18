package org.fleetranger.rc

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent

/** Starts uploading again after the controller boots, and after an update of the app. */
class BootReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        if (intent.action == Intent.ACTION_BOOT_COMPLETED || intent.action == Intent.ACTION_MY_PACKAGE_REPLACED) {
            UploadService.start(context)
        }
    }
}
