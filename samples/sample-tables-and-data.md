---
title: "Markdown Table Rendering & Round-Trip Stress Test"
category: "Benchmarks & Data"
rows: 25
columns: 6
---

# Table Rendering & Round-Trip Stress Test

This document stresses Markdown table rendering, column alignments, table borders, overflow handling, and bidirectional round-trip conversion (`Markdown -> HTML (Marked) -> Turndown GFM -> Markdown`).

## 1. Wide Financial Matrix (7 Columns, Mixed Alignment)

| Asset ID | Asset Name | Category | Current Price | 24h Change | Market Cap | Status |
| :--- | :--- | :---: | ---: | ---: | ---: | :---: |
| `BTC` | Bitcoin | *Cryptocurrency* | $64,280.50 | `+2.45%` | $1,265,400,000 | **Active** |
| `ETH` | Ethereum | *Smart Contracts* | $3,450.12 | `-0.82%` | $414,200,000 | **Active** |
| `AAPL` | Apple Inc. | *Consumer Tech* | $182.63 | `+0.15%` | $2,810,000,000 | **Active** |
| `GOOGL`| Alphabet Inc. | *Internet & AI* | $175.40 | `+1.85%` | $2,190,000,000 | **Active** |
| `MSFT` | Microsoft Corp. | *Cloud & OS* | $420.55 | `+0.92%` | $3,120,000,000 | **Active** |
| `NVDA` | NVIDIA Corp. | *Semiconductors* | $128.90 | `+4.10%` | $3,170,000,000 | **Active** |
| `AMZN` | Amazon.com | *E-commerce* | $185.20 | `-1.20%` | $1,920,000,000 | **Active** |

---

## 2. Platform Comparison Matrix (Rich Formatting Inside Cells)

| OS Platform | Shell Tech | Web Engine | Native Menus | Printing Support | Binary Target | Test Status |
| :--- | :--- | :--- | :---: | :---: | :--- | :---: |
| **macOS 13+** | Swift 5 / AppKit | `WKWebView` | `NSMenu` | `NSPrintOperation` | `.app` / `.dmg` | [Pass](https://github.com) |
| **Linux (Arch/Fedora/Ubuntu)** | Rust 2024 / Tauri v2 | `WebKitGTK 6.0` | In-Window / GTK | `WebKitPrint` (Cairo) | `.AppImage` / `.deb` | [Pass](https://github.com) |
| **Windows 10/11** | Rust 2024 / Tauri v2 | `WebView2` | Win32 Menu | System Print Dialog | `.exe` / `.msi` | Pending |

---

## 3. Large Dataset (20 Consecutive Rows)

| Index | Identifier | SHA-256 Checksum (Truncated) | Byte Size | MIME Type | Verification |
| ---: | :--- | :--- | ---: | :--- | :---: |
| 1 | `file-001.bin` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4` | 0 B | `application/octet-stream` | [x] Valid |
| 2 | `index.html` | `a1b2c3d4e5f60718293a4b5c6d7e8f9012345678` | 3,362 B | `text/html; charset=utf-8` | [x] Valid |
| 3 | `editor.js` | `f0e1d2c3b4a5968778695a4b3c2d1e0f1a2b3c4d` | 7,441 B | `text/javascript` | [x] Valid |
| 4 | `style.css` | `0123456789abcdef0123456789abcdef01234567` | 7,146 B | `text/css` | [x] Valid |
| 5 | `marked.min.js` | `49f1b2c3d4e5061728394a5b6c7d8e9f01234567` | 42,890 B | `text/javascript` | [x] Valid |
| 6 | `purify.min.js` | `88c2b1a0f9e8d7c6b5a4938271605f4e3d2c1b0a` | 21,340 B | `text/javascript` | [x] Valid |
| 7 | `turndown.js` | `11223344556677889900aabbccddeeff00112233` | 18,920 B | `text/javascript` | [x] Valid |
| 8 | `turndown-gfm.js`| `99887766554433221100ffeeddccbbaa33221100` | 8,450 B | `text/javascript` | [x] Valid |
| 9 | `cargo.lock` | `aabbccddeeff00112233445566778899aabbccdd` | 154,230 B | `text/plain` | [x] Valid |
| 10 | `tauri.conf.json`| `44556677889900aabbccddeeff00112233445566` | 1,093 B | `application/json` | [x] Valid |
| 11 | `lib.rs` | `ccddeeff00112233445566778899aabbccddeeff` | 15,200 B | `text/rust` | [x] Valid |
| 12 | `main.rs` | `1234567890abcdef1234567890abcdef12345678` | 177 B | `text/rust` | [x] Valid |
| 13 | `app-icon.png` | `fedcba0987654321fedcba0987654321fedcba09` | 7,216 B | `image/png` | [x] Valid |
| 14 | `icon.ico` | `abcdef1234567890abcdef1234567890abcdef12` | 32,450 B | `image/x-icon` | [x] Valid |
| 15 | `icon.icns` | `7890abcdef1234567890abcdef1234567890abcd` | 128,400 B | `image/x-icns` | [x] Valid |
| 16 | `package.json` | `34567890abcdef1234567890abcdef1234567890` | 498 B | `application/json` | [x] Valid |
| 17 | `package-lock` | `90abcdef1234567890abcdef1234567890abcdef` | 24,178 B | `application/json` | [x] Valid |
| 18 | `build.sh` | `ef1234567890abcdef1234567890abcdef123456` | 1,657 B | `application/x-sh` | [x] Valid |
| 19 | `prepare-vendor`| `567890abcdef1234567890abcdef1234567890ab` | 640 B | `text/javascript` | [x] Valid |
| 20 | `test.js` | `cdef1234567890abcdef1234567890abcdef1234` | 3,824 B | `text/javascript` | [x] Valid |

---

*End of Tables and Data Stress Test.*
