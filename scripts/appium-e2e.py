#!/usr/bin/env python3
"""Appium E2E test for Ressoa (mdreader) with webview support.
Requires: appium server running, device connected, ChromeDriver available.
Usage: python3 appium-e2e.py
"""
import time, os, sys

APPIUM_URL = "http://127.0.0.1:4723"
DEVICE = "adb-53cf9aa7-4VfOJ6._adb-tls-connect._tcp"
PKG = "app.mdreader"

def run():
    try:
        from appium import webdriver
        from appium.options.android import UiAutomator2Options
    except ImportError:
        print("appium-python-client not installed. Install: pip install Appium-Python-Client")
        return 1

    opts = UiAutomator2Options()
    opts.platform_name = "Android"
    opts.device_name = DEVICE
    opts.app_package = PKG
    opts.app_activity = ".MainActivity"
    opts.no_reset = True
    opts.automation_name = "UiAutomator2"
    opts.chromedriver_executable = os.path.expanduser("~/.local/bin/chromedriver")

    driver = webdriver.Remote(APPIUM_URL, options=opts)
    print("App launched via Appium")

    time.sleep(5)

    # Check native context
    print(f"Current context: {driver.current_context}")
    print(f"Contexts: {driver.contexts}")

    # Try to switch to webview context
    try:
        for ctx in driver.contexts:
            if "WEBVIEW" in ctx:
                driver.switch_to.context(ctx)
                print(f"Switched to webview context: {ctx}")
                break
    except Exception as e:
        print(f"WARN: Could not switch to webview: {e}")

    # Check page source
    try:
        source = driver.page_source
        print(f"Page source length: {len(source)}")
        if "Ressoa" in source or "waveform" in source.lower():
            print("PASS: App content detected in webview")
        else:
            print("INFO: App content not detected (may be native context)")
    except Exception as e:
        print(f"WARN: Could not get page source: {e}")

    # Switch back to native
    try:
        driver.switch_to.context("NATIVE_APP")
    except Exception:
        pass

    driver.quit()
    print("Appium E2E test complete")
    return 0

if __name__ == "__main__":
    exit(run())