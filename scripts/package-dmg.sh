#!/bin/zsh
set -euo pipefail
cd "${0:A:h}/.."

MODE="${1:-local}"
TOOL_ARCH=()
if /usr/bin/arch -arm64 /usr/bin/true 2>/dev/null; then
  TOOL_ARCH=(/usr/bin/arch -arm64)
fi
if [[ "$MODE" != local && "$MODE" != release ]]; then
  echo "Usage: $0 [local|release]" >&2
  exit 2
fi
if [[ "$MODE" == release ]]; then
  : "${MARKDOWNDESK_SIGN_IDENTITY:?Set a Developer ID Application signing identity}"
  : "${MARKDOWNDESK_NOTARY_PROFILE:?Set a notarytool Keychain profile name}"
fi

BUILD_DIR="${PWD}/build/installer"
APP="$BUILD_DIR/MarkdownDesk.app"
mkdir -p "$BUILD_DIR" dist

MARKDOWNDESK_BUILD_DIR="$BUILD_DIR" npm run build
MARKDOWNDESK_BUILD_DIR="$BUILD_DIR" npm test

VERSION=$(/usr/libexec/PlistBuddy -c 'Print CFBundleShortVersionString' "$APP/Contents/Info.plist")
ARCHS=$("${TOOL_ARCH[@]}" /usr/bin/lipo -archs "$APP/Contents/MacOS/MarkdownDesk")
if [[ "$ARCHS" == *arm64* && "$ARCHS" == *x86_64* ]]; then
  ARCH_NAME=universal
elif [[ "$ARCHS" == *arm64* ]]; then
  ARCH_NAME=arm64
else
  ARCH_NAME=x86_64
fi

if [[ "$MODE" == release ]]; then
  "${TOOL_ARCH[@]}" /usr/bin/codesign --force --options runtime --timestamp \
    --sign "$MARKDOWNDESK_SIGN_IDENTITY" "$APP"
  "${TOOL_ARCH[@]}" /usr/bin/codesign --verify --strict --verbose=2 "$APP"
  DMG="${PWD}/dist/MarkdownDesk-${VERSION}-macos-${ARCH_NAME}.dmg"
else
  DMG="${PWD}/dist/MarkdownDesk-${VERSION}-macos-${ARCH_NAME}-local-unsigned.dmg"
fi

STAGING=$(mktemp -d "$BUILD_DIR/dmg-stage.XXXXXX")
trap 'rm -rf "$STAGING"' EXIT
/bin/cp -R "$APP" "$STAGING/MarkdownDesk.app"
ln -s /Applications "$STAGING/Applications"
"${TOOL_ARCH[@]}" /usr/bin/hdiutil create -volname "MarkdownDesk ${VERSION}" \
  -srcfolder "$STAGING" -format UDZO -ov "$DMG"
"${TOOL_ARCH[@]}" /usr/bin/hdiutil verify "$DMG"

if [[ "$MODE" == release ]]; then
  "${TOOL_ARCH[@]}" /usr/bin/xcrun notarytool submit "$DMG" \
    --keychain-profile "$MARKDOWNDESK_NOTARY_PROFILE" --wait \
    --output-format json > "$BUILD_DIR/notarization.json"
  /usr/bin/python3 - "$BUILD_DIR/notarization.json" <<'PY'
import json, sys
with open(sys.argv[1], encoding='utf-8') as stream:
    result = json.load(stream)
print('Notarization:', result.get('status', 'unknown'))
if result.get('status') != 'Accepted':
    raise SystemExit('Notarization was not accepted; do not distribute this disk image.')
PY
  "${TOOL_ARCH[@]}" /usr/bin/xcrun stapler staple "$DMG"
  "${TOOL_ARCH[@]}" /usr/bin/xcrun stapler validate "$DMG"
fi

echo "$DMG"
