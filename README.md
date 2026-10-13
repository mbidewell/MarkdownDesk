# MarkdownDesk

An open-source, Mac-first Markdown editor with a single rich-text page, a Markdown source view, printing, and PDF export. Markdown files stay plain text and remain usable from the CLI.

This is an early prototype.

## Run on macOS

Requirements: macOS 13 or later, Apple Command Line Tools (or Xcode), Node.js 20 or later, and npm. The app itself needs no network connection or Node.js after it has been built.

```sh
npm ci
npm run build
open build/MarkdownDesk.app
```

Create a blank document with **New** or **⌘N**, write your notes, then choose **Save** or **⌘S** to name the Markdown file and choose its folder. Open an existing file through **File → Open** or from the Terminal:

```sh
open -a "$PWD/build/MarkdownDesk.app" "/absolute/path/to/note.md"
```

Use **File → Export PDF** for a paginated PDF, or **File → Print** for the macOS print dialog. Toggle the Markdown source view with **⌘⇧M**. The toolbar groups paragraph style, emphasis, blocks, and links; **New**, **Open**, and **Save** are always visible above it.

If the local Swift compiler and selected macOS SDK are mismatched, set `MARKDOWNDESK_SDK` to an installed compatible SDK path before building. The build script automatically chooses the macOS 26 SDK for Swift 6.3 Command Line Tools when available.

## Current scope

- Opens and saves UTF-8 `.md`, `.markdown`, and plain-text files.
- Rich view displays headings, emphasis, links, lists, tables, task lists, code blocks, quotes, images, and horizontal rules.
- YAML front matter is retained during rich edits. A file saved without editing keeps its original Markdown text.
- The source view supports exact Markdown editing, including syntax the rich view does not handle.
- The app is an unsigned local build. No installer, notarization, auto-update, or cross-platform release is included.

Rich-text editing converts the document through HTML when saving. This can normalize whitespace, Markdown syntax choices, or advanced constructs such as footnotes, math, diagrams, embedded HTML, and nested tables. For those documents, edit in source view. Keep your files under version control or backed up while using this prototype.

The build and automated Markdown tests pass. Opening files from the toolbar and the CLI, saving a rich-text edit, and exporting one-page and two-page PDFs were checked in the Mac app. Print uses the same AppKit document view as PDF export.

## Develop

```sh
npm ci
npm run build
npm test
```

The native shell is written in Swift/AppKit and uses WebKit. Browser-side Markdown conversion uses [Marked](https://marked.js.org/), [DOMPurify](https://github.com/cure53/DOMPurify), [Turndown](https://github.com/mixmark-io/turndown), and [Turndown GFM](https://github.com/mixmark-io/turndown/tree/master/packages/turndown-plugin-gfm). Their license files are bundled in `Contents/Resources/Licenses`.

Contributions are welcome. Please open an issue before starting a large feature; the document model and lossless editing approach are still evolving.
