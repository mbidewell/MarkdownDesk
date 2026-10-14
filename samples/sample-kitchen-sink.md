---
title: "MarkdownDesk Kitchen Sink Stress Test"
author: "MarkdownDesk Test Suite"
date: "2026-10-14"
version: 1.0.4
status: draft
tags:
  - markdown
  - stress-test
  - typography
  - gfm
description: "A comprehensive document designed to stress-test rendering, font sizing, line heights, lists, tables, and block structures."
---

# 1. Typography & Hierarchy (H1)

This paragraph tests standard body typography using **bold text**, *italic text*, ***bold italic text***, ~~strikethrough~~, and `inline code spans`. We also test [hyperlinks to external sites](https://example.com "Example Link Title") and [internal anchor links](#tables--complex-data-h2).

The typography should respect comfortable line heights (1.73 in the editor), proportional margin collapsing, and warm paper ink colors (`#282c29`).

## 1.1 Secondary Heading (H2)

Here is a paragraph preceding a tertiary heading. Let's make sure margins above and below headings don't cause awkward page overflow or excessive gaps.

### 1.1.1 Tertiary Heading (H3)

#### 1.1.1.1 Quaternary Heading (H4)

##### Quinary Heading (H5)

###### Senary Heading (H6)

---

# 2. Lists & Hierarchies

### Unordered Lists (Multi-level & Loose)
* Level 1 item with **bold prefix**: Notice the bullet alignment.
  * Level 2 nested item with *italics*.
    * Level 3 nested item with `inline monospace code`.
      * Level 4 deeply nested item.
* Another Level 1 item with multiple sentences. This item has enough text to wrap across multiple lines to verify that indentation hangs correctly under the bullet point rather than wrapping under the bullet glyph.
* Third Level 1 item.

### Ordered Lists
1. First step in the initialization process.
2. Second step with nested sub-steps:
   1. Sub-step A: Verify environment variables (`MARKDOWNDESK_SDK`).
   2. Sub-step B: Compile native wrapper with `swiftc` or `cargo`.
   3. Sub-step C: Mount AppDir or bundle `.dmg`.
3. Third step: Run automated regression suite via `npm test`.

### GFM Task Lists / Checkboxes
- [x] Initial prototype designed and tested on macOS AppKit
- [x] Web frontend decoupled from platform-specific WebKit APIs
- [x] Linux host shell implemented with Tauri v2 and WebKitGTK
- [x] Standalone Linux AppImage built and verified
- [ ] Add dark mode theme support
- [ ] Implement spellcheck language selection

---

# 3. Blockquotes & Callouts

> "Markdown is intended to be as easy-to-read and easy-to-write as is feasible."
>
> — John Gruber

Nested blockquote test:

> Level 1 quote: Design is not just what it looks like and feels like.
>
> > Level 2 quote: Design is how it works.
> >
> > > Level 3 quote: Simplicity is about subtracting the obvious and adding the meaningful.
>
> Back to Level 1 with some `inline code` and a [link](https://daringfireball.net).

---

# 4. Code Blocks & Syntax

### JavaScript / Web Engine Interop
```javascript
function initEditor(options = {}) {
  const editor = document.getElementById('editor');
  const source = document.getElementById('source');
  
  // Test unicode inside comments: 🚀 ✨ 📝
  const isMac = /Mac/i.test(navigator.userAgent);
  console.log(`Platform: ${isMac ? 'Apple macOS' : 'Linux / Unix'}`);
  
  return {
    getWordCount: () => editor.innerText.trim().split(/\s+/).length,
    isDirty: () => Boolean(options.dirty),
  };
}
```

### Rust Tauri v2 Backend Command
```rust
#[tauri::command]
fn save_document(path: Option<String>, content: String) -> Result<String, String> {
    let target = path.unwrap_or_else(|| "Untitled.md".to_string());
    std::fs::write(&target, &content)
        .map_err(|e| format!("IO error: {}", e))?;
    println!("Successfully wrote {} bytes to {}", content.len(), target);
    Ok(target)
}
```

### Raw Shell Snippet
```bash
# Build the AppImage on Linux
npm ci
npm run appimage
./src-tauri/target/release/bundle/appimage/MarkdownDesk_0.1.0_amd64.AppImage
```

---

# 5. Tables & Complex Data

| Feature | macOS AppKit | Linux Tauri v2 | Windows WebView2 | Status |
| :--- | :---: | :---: | :---: | ---: |
| **Window Engine** | AppKit (`NSWindow`) | GTK3/4 / TAO | Win32 / WinUI | `100% Native` |
| **Browser Runtime** | Apple WebKit | WebKitGTK 6.0 | Edge WebView2 | `Active` |
| **Executable Size** | ~3.8 MB | ~12.4 MB | ~10.5 MB | `Optimized` |
| **Offline First** | Yes | Yes | Yes | `Verified` |
| **PDF Export** | `NSPrintOperation` | `WebKitPrint` / Cairo | `PrintToPDF` | `Working` |

---

# 6. Unicode, Emojis, & Math Symbols

* **Currency symbols**: $100.00 | €85.50 | £72.25 | ¥12,500 | ₹7,500 | ₿0.0042
* **Mathematical Greek**: $\alpha, \beta, \gamma, \Delta, \pi \approx 3.14159, \sum_{i=1}^n i = \frac{n(n+1)}{2}$
* **Arrows & Logic**: $\leftarrow \rightarrow \uparrow \downarrow \leftrightarrow \Rightarrow \forall x \in \mathbb{R} \exists y$
* **Emojis**: 🚀 ⚡ 📦 🎨 💻 🐧 🍎 🪟 📖 🖋️ 💡 ✅ ❌ ⚠️
* **International Alphabets**:
  * Japanese: こんにちは世界 (Hello World)
  * Korean: 안녕하세요 세계
  * German: Schöne Grüße aus der Bärenstraße (ä, ö, ü, ß)
  * French: Éléphant, naïve, château, français
  * Greek: Καλημέρα κόσμε

---

# 7. Images & Horizontal Separators

An inline test image with alt text and title:

![MarkdownDesk Icon](https://raw.githubusercontent.com/mbidewell/MarkdownDesk/crossplatform/app-icon.png "MarkdownDesk logo")

***

*End of Kitchen Sink Stress Test Document.*
