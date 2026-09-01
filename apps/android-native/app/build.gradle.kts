import java.util.Properties

plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
    id("org.jetbrains.kotlin.plugin.compose")
}

// One tester APK for current Android phones plus the common x86_64 emulator
// lane. The previously distributed arm64-only APK installed on the owner's
// test phone, confirming that device is 64-bit; x86_64 keeps device testing
// possible without platform translation.
val nativeAbis = listOf("arm64-v8a", "x86_64")
val pinnedNdkVersion = "27.2.12479018"
val minimumLoadSegmentAlignment = 0x4000L
val androidRustLinkFlags = listOf(
    "-C link-arg=-Wl,-z,max-page-size=16384",
    // oboe-sys 0.6.1 emits c++_static but omits its C++ ABI companion,
    // leaving __gxx_personality_v0 unresolved until the APK is opened.
    // Keep both at the end of the final link so the bridge is self-contained.
    "-C link-arg=-lc++_static",
    "-C link-arg=-lc++abi",
).joinToString(" ")

val resolveConfiguredNdkDir = {
    System.getenv("ANDROID_NDK_HOME")
        ?: run {
            val properties = Properties()
            val localProperties = rootProject.file("local.properties")
            if (localProperties.isFile) {
                localProperties.inputStream().use { properties.load(it) }
            }
            val sdkDir = properties.getProperty("sdk.dir") ?: System.getenv("ANDROID_HOME")
            val pinnedNdk = sdkDir?.let { File(it, "ndk/$pinnedNdkVersion") }
            pinnedNdk
                ?.takeIf { it.isDirectory }
                ?.absolutePath
                ?: sdkDir
                    ?.let { File(it, "ndk").listFiles() }
                    ?.filter { it.isDirectory }
                    ?.maxByOrNull { it.name }
                    ?.absolutePath
        }
}

val findLlvmReadelf = { ndkDir: String ->
    val executableName = if (System.getProperty("os.name").lowercase().contains("windows")) {
        "llvm-readelf.exe"
    } else {
        "llvm-readelf"
    }
    File(ndkDir, "toolchains/llvm/prebuilt")
        .listFiles()
        ?.map { File(it, "bin/$executableName") }
        ?.firstOrNull { it.isFile }
}

android {
    namespace = "com.yesmaster.app"
    compileSdk = 35
    // Pin to the provisioned NDK so AGP's strip step finds llvm-strip
    // (unset, AGP wants its own default NDK and packages the .so
    // unstripped — ~5 MB of dead weight in the APK).
    ndkVersion = pinnedNdkVersion

    defaultConfig {
        applicationId = "com.yesmaster.app"
        minSdk = 29
        targetSdk = 35
        versionCode = 1
        versionName = "0.1.0"
        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
        ndk {
            // One sideloadable package covers current 64-bit phones and the
            // x86_64 emulator used by the connected bridge-load gate.
            abiFilters += nativeAbis
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
    testOptions {
        unitTests {
            // Compose semantics lane: Robolectric renders real composables on
            // the JVM, which needs Android resources on the test classpath.
            // Chosen lane (U19): JVM/Robolectric is the automated gate — it
            // runs on any dev machine and in CI's existing `gradlew test` with
            // no emulator; a physical-device pass stays an explicit
            // pre-release gate (U20), never implied by this lane.
            isIncludeAndroidResources = true
        }
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

// Build the Rust bridge (.so) with cargo-ndk before jniLibs are merged into
// the APK. Release profile: the debug .so is ~165 MB; release is what ships.
//
// Portability (adversarial-review findings): cargo/cargo-ndk are resolved
// via ~/.cargo/bin without relying on the Gradle daemon's inherited PATH
// (IDE launches on macOS don't source shell profiles), and the NDK is
// resolved from ANDROID_NDK_HOME or local.properties' sdk.dir (newest
// ndk/<version>). Inputs/outputs are declared so a no-change build is
// UP-TO-DATE instead of respawning cargo every invocation.
val cargoNdk = tasks.register<Exec>("cargoNdk") {
    workingDir = file("../rust")
    inputs.dir(file("../rust/src"))
    inputs.files(file("../rust/Cargo.toml"), file("../rust/Cargo.lock"))
    // The .so statically links the facade + engine — their sources are
    // inputs too, so touching shared Rust re-triggers this task. Manifests,
    // the engine's build script, and the config it embeds count as well: a
    // feature edit or tauri.conf change that touches no src/ file must not
    // leave a stale .so reported UP-TO-DATE (adversarial-review finding).
    inputs.dir(file("../../iphone-native/rust/src"))
    inputs.files(file("../../iphone-native/rust/Cargo.toml"))
    inputs.dir(file("../../../src-tauri/src"))
    inputs.files(
        file("../../../src-tauri/Cargo.toml"),
        file("../../../src-tauri/build.rs"),
        file("../../../src-tauri/tauri.conf.json"),
    )
    inputs.dir(file("../../../src-tauri/capabilities"))
    outputs.dir(file("src/main/jniLibs"))

    doFirst {
        // jniLibs is generated and ignored. Clear it whenever this task runs
        // so removing an ABI or cdylib cannot leave a stale library packaged
        // or evaluated by the verification tasks below.
        project.delete(file("src/main/jniLibs"))
    }

    val pathSeparator = File.pathSeparator
    val cargoBinDir = File(System.getProperty("user.home"), ".cargo/bin")
    val rustupCargo = File(System.getProperty("user.home"), ".rustup/toolchains")
        .listFiles()
        ?.filter { it.isDirectory && it.name.startsWith("stable-") }
        ?.sortedByDescending { it.lastModified() }
        ?.flatMap { toolchain ->
            sequenceOf("cargo.exe", "cargo").map { File(toolchain, "bin/$it") }.toList()
        }
        ?.firstOrNull { it.isFile }
    val cargoExe = sequenceOf("cargo.exe", "cargo")
        .map { File(cargoBinDir, it) }
        .firstOrNull { it.isFile }
        ?.absolutePath
        ?: rustupCargo?.absolutePath
        ?: "cargo"
    val cargoToolchainBin = File(cargoExe).parentFile
    val cargoSearchPath = listOfNotNull(
        cargoBinDir.takeIf { it.isDirectory }?.absolutePath,
        cargoToolchainBin?.takeIf { it.isDirectory }?.absolutePath,
        System.getenv("PATH"),
    ).joinToString(pathSeparator)
    if (cargoSearchPath.isNotBlank()) {
        environment("PATH", cargoSearchPath)
    }

    val ndkDir = resolveConfiguredNdkDir()
    if (ndkDir != null) {
        environment("ANDROID_NDK_HOME", ndkDir)
    }

    // Exec tasks do not fingerprint commandLine/environment, so an NDK or
    // cargo swap would otherwise leave the task UP-TO-DATE with a .so built
    // by the old toolchain.
    inputs.property("ndkDir", ndkDir ?: "")
    inputs.property("cargo", cargoExe)
    inputs.property("androidRustLinkFlags", androidRustLinkFlags)

    // Keep every ABI explicit so the package remains loadable on newer 16 KB
    // page-size devices and the alignment gate below has one rule for all
    // libraries.
    listOf(
        "AARCH64_LINUX_ANDROID",
        "X86_64_LINUX_ANDROID",
    ).forEach { target ->
        environment("CARGO_TARGET_${target}_RUSTFLAGS", androidRustLinkFlags)
    }

    commandLine(
        listOf(cargoExe, "ndk") +
            nativeAbis.flatMap { listOf("-t", it) } +
            listOf(
                // Must match minSdk: cargo-ndk defaults to API 21, whose sysroot
                // predates libaaudio (API 26+) — the audition link fails without it.
                "--platform", "29",
                "-o", file("src/main/jniLibs").absolutePath,
                "build", "--release",
            )
    )
}

val verifyNativeLoadAlignment = tasks.register("verifyNativeLoadAlignment") {
    dependsOn(cargoNdk)
    val nativeLibs = fileTree("src/main/jniLibs") {
        include("**/*.so")
    }
    inputs.files(nativeLibs)
    inputs.property("minimumLoadSegmentAlignment", minimumLoadSegmentAlignment)

    doLast {
        val ndkDir = resolveConfiguredNdkDir()
            ?: throw GradleException("ANDROID_NDK_HOME, local.properties sdk.dir, or ANDROID_HOME must point to NDK $pinnedNdkVersion")
        val readelf = findLlvmReadelf(ndkDir)
            ?: throw GradleException("llvm-readelf not found under $ndkDir")
        val loadLine = Regex("""^\s*LOAD\s+.*\s(0x[0-9a-fA-F]+)\s*$""")
        val failures = mutableListOf<String>()

        nativeLibs.files.sortedBy { it.absolutePath }.forEach { lib ->
            val output = providers.exec {
                commandLine(readelf.absolutePath, "--program-headers", "--wide", lib.absolutePath)
            }.standardOutput.asText.get()
            val alignments = output
                .lineSequence()
                .mapNotNull { loadLine.find(it)?.groupValues?.get(1) }
                .toList()
            if (alignments.isEmpty()) {
                failures += "${lib.name}: no LOAD segments reported"
                return@forEach
            }
            alignments
                .map { it to it.removePrefix("0x").toLong(16) }
                .filter { (_, alignment) -> alignment < minimumLoadSegmentAlignment }
                .forEach { (raw, _) -> failures += "${lib.name}: LOAD segment align=$raw" }
        }

        if (failures.isNotEmpty()) {
            throw GradleException(
                "Native libraries must use LOAD segment alignment >= 0x${minimumLoadSegmentAlignment.toString(16)}:\n" +
                    failures.joinToString("\n")
            )
        }
    }
}

val verifyNativeBridgeCoverage = tasks.register("verifyNativeBridgeCoverage") {
    dependsOn(cargoNdk)
    val bridgeName = "libyes_master_android_bridge.so"
    val expected = nativeAbis.map { abi -> file("src/main/jniLibs/$abi/$bridgeName") }
    inputs.files(expected)

    doLast {
        val missing = expected.filterNot { it.isFile && it.length() > 0 }
        if (missing.isNotEmpty()) {
            throw GradleException(
                "Tester APK is missing the native audio bridge for: " +
                    missing.joinToString { it.parentFile.name }
            )
        }
    }
}

verifyNativeLoadAlignment.configure {
    dependsOn(verifyNativeBridgeCoverage)
}

// Hook into jniLib merging only — APK packaging needs the .so, but the JVM
// unit-test lane (the wire drift gate's fourth consumer) must stay runnable
// on a machine with no Rust or NDK installed.
tasks.matching { it.name.matches(Regex("merge.*JniLibFolders")) }.configureEach {
    dependsOn(verifyNativeLoadAlignment)
}

dependencies {
    val composeBom = platform("androidx.compose:compose-bom:2024.10.01")
    implementation(composeBom)
    implementation("androidx.activity:activity-compose:1.9.3")
    implementation("androidx.core:core-ktx:1.15.0")
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
    // Compose semantics lane (see testOptions above).
    testImplementation(composeBom)
    testImplementation("androidx.compose.ui:ui-test-junit4")
    // debugImplementation, not testImplementation: the rule's host activity
    // must be merged into the debug manifest for Robolectric to resolve it.
    // The release APK (what ships) is untouched.
    debugImplementation("androidx.compose.ui:ui-test-manifest")
    testImplementation("org.robolectric:robolectric:4.14.1")
    androidTestImplementation("androidx.test.ext:junit:1.2.1")
    androidTestImplementation("androidx.test:runner:1.6.2")
}
