plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
    id("org.jetbrains.kotlin.plugin.compose")
}

android {
    namespace = "com.yesmaster.app"
    compileSdk = 35

    defaultConfig {
        applicationId = "com.yesmaster.app"
        minSdk = 29
        targetSdk = 35
        versionCode = 1
        versionName = "0.1.0"
        ndk {
            // Sideload MVP targets real hardware; emulator x86_64 can be
            // added to this list when wanted.
            abiFilters += listOf("arm64-v8a")
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = false
        }
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
    kotlinOptions {
        jvmTarget = "17"
    }
    buildFeatures {
        compose = true
    }
    packaging {
        jniLibs {
            // cargo-ndk copies every cdylib in the dependency graph, but the
            // android bridge links the engine statically (DT_NEEDED is only
            // libc/libm/libdl) — the other two are ~10 MB of dead weight.
            excludes += listOf(
                "**/libyes_master_iphone_native_bridge.so",
                "**/libyes_master_lib.so",
            )
        }
    }
}

// Build the Rust bridge (.so) with cargo-ndk before every Android build so
// jniLibs is always current with the engine. Release profile: the debug .so
// is ~165 MB; release is what ships in the APK.
val cargoNdk = tasks.register<Exec>("cargoNdk") {
    workingDir = file("../rust")
    commandLine(
        "cargo", "ndk",
        "-t", "arm64-v8a",
        "-o", file("src/main/jniLibs").absolutePath,
        "build", "--release",
    )
}

tasks.named("preBuild") {
    dependsOn(cargoNdk)
}

dependencies {
    val composeBom = platform("androidx.compose:compose-bom:2024.10.01")
    implementation(composeBom)
    implementation("androidx.activity:activity-compose:1.9.3")
    implementation("androidx.compose.material3:material3")
    implementation("androidx.compose.ui:ui")
    implementation("androidx.compose.ui:ui-tooling-preview")
    implementation("androidx.lifecycle:lifecycle-viewmodel-compose:2.8.7")
    implementation("androidx.lifecycle:lifecycle-runtime-ktx:2.8.7")
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-android:1.9.0")
    // Wire decode: plain Gson (no codegen, runs in JVM unit tests so the
    // wire-samples drift gate gets a fourth consumer).
    implementation("com.google.code.gson:gson:2.11.0")

    testImplementation("junit:junit:4.13.2")
}
