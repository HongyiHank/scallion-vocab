#!/usr/bin/env python3
"""
Patch AndroidManifest.xml: add REQUEST_INSTALL_PACKAGES permission + FileProvider.

Usage:
    python3 scripts/lib/patch-manifest.py <path-to-AndroidManifest.xml>
"""

import sys

path = sys.argv[1]
with open(path, 'r') as f:
    content = f.read()

# Add permission after INTERNET
perm = '<uses-permission android:name="android.permission.REQUEST_INSTALL_PACKAGES"/>'
internet_line = '<uses-permission android:name="android.permission.INTERNET" />'
if perm not in content and internet_line in content:
    content = content.replace(internet_line, internet_line + '\n    ' + perm, 1)
elif perm not in content:
    content = content.replace('<application', '    ' + perm + '\n    <application', 1)

# Add FileProvider before </application>
provider_block = '''    <provider
        android:name="androidx.core.content.FileProvider"
        android:authorities="${applicationId}.fileprovider"
        android:exported="false"
        android:grantUriPermissions="true">
        <meta-data
            android:name="android.support.FILE_PROVIDER_PATHS"
            android:resource="@xml/provider_paths" />
    </provider>'''
if 'FileProvider' not in content:
    content = content.replace('</application>', provider_block + '\n</application>', 1)

with open(path, 'w') as f:
    f.write(content)
