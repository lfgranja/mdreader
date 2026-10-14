pluginManagement {
    repositories {
        google()
        mavenCentral()
        gradlePluginPortal()
    }
    plugins {
        id("com.android.application") version "9.3.1"
        id("org.jetbrains.kotlin.android") version "2.2.10"
    }
}

rootProject.name = "focus-injector"
include(":app")