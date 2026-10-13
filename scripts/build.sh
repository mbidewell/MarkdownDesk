#!/bin/zsh
set -euo pipefail
cd "${0:A:h}/.."
APP="${PWD}/build/MarkdownDesk.app"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources/Web/vendor" "$APP/Contents/Resources/Licenses"
cp Info.plist "$APP/Contents/Info.plist"
cp Web/index.html Web/style.css Web/editor.js "$APP/Contents/Resources/Web/"
cp node_modules/marked/lib/marked.umd.js "$APP/Contents/Resources/Web/vendor/marked.js"
cp node_modules/dompurify/dist/purify.min.js "$APP/Contents/Resources/Web/vendor/purify.js"
cp node_modules/turndown/dist/turndown.js "$APP/Contents/Resources/Web/vendor/turndown.js"
cp node_modules/turndown-plugin-gfm/dist/turndown-plugin-gfm.js "$APP/Contents/Resources/Web/vendor/turndown-gfm.js"
for name in marked dompurify turndown turndown-plugin-gfm; do
  cp "node_modules/$name/LICENSE" "$APP/Contents/Resources/Licenses/$name.txt"
done
SDK="${MARKDOWNDESK_SDK:-$(xcrun --show-sdk-path)}"
if [[ -z "${MARKDOWNDESK_SDK:-}" && "$(swiftc --version 2>/dev/null)" == *"Swift version 6.3"* \
      && -d /Library/Developer/CommandLineTools/SDKs/MacOSX26.sdk ]]; then
  SDK=/Library/Developer/CommandLineTools/SDKs/MacOSX26.sdk
fi
swiftc -swift-version 5 -O -sdk "$SDK" \
  -module-cache-path "${PWD}/build/ModuleCache" -framework AppKit -framework WebKit \
  Sources/MarkdownDesk.swift -o "$APP/Contents/MacOS/MarkdownDesk"
echo "$APP"
