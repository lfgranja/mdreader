# Add project specific ProGuard rules here.
# You can control the set of applied configuration files using the
# proguardFiles setting in build.gradle.
#
# For more details, see
#   http://developer.android.com/guide/developing/tools/proguard.html

# If your project uses WebView with JS, uncomment the following
# and specify the fully qualified class name to the JavaScript interface
# class:
#-keepclassmembers class fqcn.of.javascript.interface.for.webview {
#   public *;
#}

# Uncomment this to preserve the line number information for
# debugging stack traces.
#-keepattributes SourceFile,LineNumberTable

# If you keep the line number information, uncomment this to
# hide the original source file name.
#-renamesourcefileattribute SourceFile

# JNI entry points called from Rust by name (find_class +
# call_static_method in src-tauri/src/shell_jni.rs). R8 cannot see these
# references, so without this rule the release build strips/renames the
# method and the publish path dies with NoSuchMethodError at runtime
# (observed 2026-10-16 on the Wave B APK, at app.mdreader.Rust.ipc).
-keep class app.mdreader.ShellHost {
    public static void publishMirror(java.lang.String, java.lang.String);
}