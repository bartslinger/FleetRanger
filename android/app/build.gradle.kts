plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlin.compose)
}

android {
    namespace = "org.fleetranger.rc"
    // Keep in step with the SDK packages in the Dockerfile. Compose 1.12 needs
    // API 37 to compile against.
    compileSdk = 37
    buildToolsVersion = "37.0.0"

    defaultConfig {
        applicationId = "org.fleetranger.rc"
        // Android 10, as on the DJI RC Pro Enterprise.
        minSdk = 29
        targetSdk = 36
        versionCode = 1
        versionName = "0.1.0"
    }

    buildFeatures {
        compose = true
        // BuildConfig.DEBUG: only debug builds accept a server over plain HTTP.
        buildConfig = true
    }
}

kotlin {
    jvmToolchain(21)
}

dependencies {
    implementation(platform(libs.compose.bom))
    implementation(libs.compose.foundation)
    implementation(libs.compose.ui)
    implementation(libs.compose.ui.tooling.preview)
    implementation(libs.activity.compose)
    implementation(libs.zxing.core)
    debugImplementation(libs.compose.ui.tooling)
}
