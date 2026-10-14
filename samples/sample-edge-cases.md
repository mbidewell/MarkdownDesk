---
title: "Markdown Edge Cases & Sanitization Stress Test"
author: "Edge Case Testing Group"
nested_metadata:
  complex: true
  levels:
    alpha: 1
    beta: [10, 20, 30]
    gamma:
      flag: "enabled"
multiline_text: >
  This is a folded multiline string in YAML front matter.
  It preserves paragraph structure across multiple lines.
array_of_objects:
  - id: 1
    name: "Entry One"
  - id: 2
    name: "Entry Two"
---

# Edge Cases & Sanitization Stress Test

This document stresses parser resilience, HTML sanitization, character escaping, deep nesting, and boundary cases.

## 1. Character Escaping & Special Sequences

Testing literal Markdown symbols using backslash escapes:
* Literal asterisks: \*not bold\* and \*\*not bold either\*\*
* Literal underscores: \_not italic\_ and \_\_not bold\_\_
* Literal brackets & parens: \[Not a link\]\(https://example.com\)
* Literal hashes: \# Not a heading
* Literal backticks: \`not inline code\`
* Literal backslashes: \\\\
* Double backticks with spaces inside: `` `code with backticks` ``

---

## 2. Deep Nesting & Mixed Block Hierarchy

> Level 1 Quote
> * Item 1 in blockquote
>   * Nested sub-item in blockquote
>     > Nested blockquote inside list inside blockquote!
>     > 
>     > > Third level nested quote!
>     > > With `code` and **bold**.
> * Item 2 in blockquote

---

## 3. HTML Sanitization & Safety Test

The editor uses `DOMPurify` to ensure that hostile code never executes:

* **Hostile script tag** (should be stripped or neutralized):
  <script>window.isHacked = true; console.error("XSS Vulnerability Executed!");</script>
* **Hostile image tag with onerror** (should have `onerror` stripped):
  <img src="invalid-image-url.jpg" onerror="alert('XSS!')" alt="Safe image fallback">
* **Hostile iframe** (should be stripped):
  <iframe src="https://example.com" width="400" height="300"></iframe>

### Safe Inline HTML Elements
* Keyboard shortcuts: Press <kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>M</kbd> to toggle source mode.
* Highlighted text: This is <mark>important text</mark> rendered with a mark tag.
* Superscript and Subscript: H<sub>2</sub>O and E = mc<sup>2</sup>.
* Abbreviations: <abbr title="GitHub Flavored Markdown">GFM</abbr> is supported.
* Deleted and inserted text: <del>Old incorrect price</del> <ins>New discounted price</ins>.

---

## 4. Word Wrap & Overflow Stress

### Extremely Long Unbroken String (Testing CSS `overflow-wrap: break-word`)
SupercalifragilisticexpialidociousAntidisestablishmentarianismPneumonoultramicroscopicsilicovolcanoconiosisPseudopseudohypoparathyroidismFloccinaucinihilipilificationHonorificabilitudinitatibusDichlorodiphenyltrichloroethane

### Very Long URL
https://subdomain.example.com/very/deeply/nested/path/to/a/resource/with/extremely/long/query/parameters?param1=value1&param2=value2&param3=value3&param4=verylongstringthatdoesnotbreakandmightoverflowthecontainerunlesswrappedproperly#section-anchor-with-extra-long-identifier

---

## 5. Consecutive Horizontal Rules & Empty Blocks

---

***

___

### End of Edge Cases Test Document
