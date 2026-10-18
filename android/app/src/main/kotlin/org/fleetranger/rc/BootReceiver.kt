package org.fleetranger.rc

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent

/** Starts uploading again after the controller boots. */
class BootReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        if (intent.action == Intent.ACTION_BOOT_COMPLETED) UploadService.start(context)
    }
}
