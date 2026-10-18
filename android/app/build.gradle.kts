plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlin.compose)
}

// Releases are tagged android-vX.Y.Z, and release.sh passes X.Y.Z; every
// other build is "dev". See "Releases" in README.md.
val releaseVersion: String? = providers.gradleProperty("releaseVersion").orNull

/** 0.2.3 as 2003: grows with every release, as Android requires for updates. */
fun versionCode(version: String): Int {
    val parts = version.split(".").map { it.toIntOrNull() ?: -1 }
    require(parts.size == 3 && parts.all { it in 0..999 }) { "expected a version like 0.1.0, got $version" }
    return parts[0] * 1_000_000 + parts[1] * 1_000 + parts[2]
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
        versionCode = releaseVersion?.let(::versionCode) ?: 1
        versionName = releaseVersion ?: "dev"
    }

    // The release key stays out of the repository: release.sh mounts it.
    signingConfigs {
        create("release") {
            storeFile = System.getenv("FLEETRANGER_KEYSTORE")?.let(::file)
            storePassword = System.getenv("FLEETRANGER_KEYSTORE_PASSWORD")
            keyAlias = "fleetranger"
            keyPassword = System.getenv("FLEETRANGER_KEYSTORE_PASSWORD")
        }
    }

    buildTypes {
        release {
            signingConfig = signingConfigs.getByName("release")
            // R8: a much smaller APK to download on the controller.
            isMinifyEnabled = true
            isShrinkResources = true
            proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt"))
        }
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

// Only tagged releases: a release build without a version is a mistake.
tasks.configureEach {
    if (name == "preReleaseBuild") {
        doFirst {
            check(releaseVersion != null) { "Build releases with release.sh, on an android-v… tag." }
        }
    }
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
