/* global marked, DOMPurify, TurndownService, turndownPluginGfm, webkit */
(() => {
  'use strict';
  const editor = document.getElementById('editor');
  const source = document.getElementById('source');
  const modeButton = document.getElementById('mode');
  const blockStyle = document.getElementById('block-style');
  const viewLabel = document.getElementById('view-label');
  const wordcount = document.getElementById('wordcount');
  const filename = document.getElementById('filename');
  const turndown = new TurndownService({ headingStyle: 'atx', codeBlockStyle: 'fenced', bulletListMarker: '-' });
  turndown.use(turndownPluginGfm.gfm);
  let initialMarkdown = '';
  let frontMatter = '';
  let changed = false;
  let savedRange;

  function splitFrontMatter(text) {
    const match = text.match(/^(---\r?\n[\s\S]*?\r?\n---\r?\n)/);
    return match ? [match[1], text.slice(match[1].length)] : ['', text];
  }

  function render(text) {
    const [front, body] = splitFrontMatter(text);
    frontMatter = front;
    const html = marked.parse(body, { gfm: true, breaks: false });
    editor.innerHTML = DOMPurify.sanitize(html, {
      USE_PROFILES: { html: true },
      ADD_ATTR: ['target'],
      FORBID_TAGS: ['style', 'iframe', 'form', 'script']
    });
    updateWordCount();
  }

  function markdown() {
    if (!changed) return initialMarkdown;
    if (document.body.classList.contains('source-mode')) return source.value;
    return frontMatter + turndown.turndown(editor.innerHTML)
      .replace(/[ \t]+(?=\n)/g, '')
      .replace(/\n{3,}/g, '\n\n')
      .trimEnd() + '\n';
  }

  function updateWordCount() {
    const text = document.body.classList.contains('source-mode') ? source.value : (editor.innerText ?? editor.textContent);
    const words = text.trim().match(/\S+/g);
    wordcount.textContent = `${words ? words.length : 0} words`;
  }

  function reportChange() {
    const firstChange = !changed;
    changed = true;
    updateWordCount();
    if (firstChange && window.webkit?.messageHandlers?.changed) webkit.messageHandlers.changed.postMessage(true);
  }

  function setMarkdown(text, name, base) {
    initialMarkdown = text;
    changed = false;
    savedRange = null;
    source.value = text;
    filename.textContent = name || 'Untitled.md';
    if (base) {
      let baseElement = document.querySelector('base');
      if (!baseElement) {
        baseElement = document.createElement('base');
        document.head.appendChild(baseElement);
      }
      baseElement.href = base;
    }
    render(text);
    document.getElementById('workspace').scrollTop = 0;
  }

  function switchMode() {
    if (document.body.classList.contains('source-mode')) {
      render(source.value);
      document.body.classList.remove('source-mode');
      modeButton.innerHTML = 'Source <span class="mode-glyph" aria-hidden="true">⌘⇧M</span>';
      viewLabel.textContent = 'RICH TEXT';
      editor.focus();
    } else {
      source.value = markdown();
      document.body.classList.add('source-mode');
      modeButton.innerHTML = 'Rich text <span class="mode-glyph" aria-hidden="true">⌘⇧M</span>';
      viewLabel.textContent = 'MARKDOWN SOURCE';
      source.focus();
    }
    updateWordCount();
  }

  function format(action) {
    if (document.body.classList.contains('source-mode')) switchMode();
    editor.focus();
    if (savedRange) {
      const selection = window.getSelection();
      selection.removeAllRanges();
      selection.addRange(savedRange);
    }
    const block = (tag) => document.execCommand('formatBlock', false, tag);
    switch (action) {
      case 'paragraph': block('p'); break;
      case 'h1': case 'h2': case 'h3': block(action); break;
      case 'bold': case 'italic': document.execCommand(action); break;
      case 'unordered': document.execCommand('insertUnorderedList'); break;
      case 'ordered': document.execCommand('insertOrderedList'); break;
      case 'quote': block('blockquote'); break;
      case 'code': block('pre'); break;
      case 'rule': document.execCommand('insertHorizontalRule'); break;
      case 'link': {
        const url = prompt('Link URL', 'https://');
        if (url && /^(https?:|mailto:|#|\.\.?\/)/i.test(url)) document.execCommand('createLink', false, url);
        break;
      }
    }
    reportChange();
  }

  document.querySelectorAll('[data-action]').forEach((button) => {
    button.addEventListener('mousedown', (event) => event.preventDefault());
    button.addEventListener('click', () => format(button.dataset.action));
  });
  blockStyle.addEventListener('change', () => format(blockStyle.value));
  document.addEventListener('selectionchange', () => {
    const selection = window.getSelection();
    if (!selection.rangeCount || !editor.contains(selection.anchorNode)) return;
    savedRange = selection.getRangeAt(0).cloneRange();
    const element = selection.anchorNode.nodeType === Node.ELEMENT_NODE
      ? selection.anchorNode : selection.anchorNode.parentElement;
    const block = element?.closest('h1, h2, h3');
    blockStyle.value = block?.tagName.toLowerCase() || 'paragraph';
  });
  document.getElementById('open-file').addEventListener('click', () => webkit.messageHandlers.command.postMessage('open'));
  document.getElementById('save-file').addEventListener('click', () => webkit.messageHandlers.command.postMessage('save'));
  modeButton.addEventListener('click', switchMode);
  editor.addEventListener('input', reportChange);
  source.addEventListener('input', reportChange);
  editor.addEventListener('click', (event) => {
    const link = event.target.closest('a');
    if (link && event.metaKey) {
      event.preventDefault();
      webkit.messageHandlers.openLink.postMessage(link.href);
    }
  });
  document.addEventListener('keydown', (event) => {
    if (event.metaKey && event.shiftKey && event.key.toLowerCase() === 'm') {
      event.preventDefault();
      switchMode();
    }
  });

  window.editorAPI = {
    setMarkdown,
    getMarkdown: markdown,
    format,
    toggleSource: switchMode,
    getPrintHTML: () => {
      if (document.body.classList.contains('source-mode')) render(source.value);
      const base = document.querySelector('base')?.outerHTML || '';
      return `<!doctype html><html><head><meta charset="utf-8">${base}<style>
        body { font-family: Georgia, serif; font-size: 12pt; color: #282c29; }
        h1, h2, h3 { font-family: Helvetica, sans-serif; color: #282c29; }
        h1 { font-size: 25pt; } h2 { font-size: 19pt; } h3 { font-size: 15pt; }
        p, li { line-height: 1.5; } blockquote { color: #5d625a; border-left: 3px solid #bc593d; padding-left: 12px; }
        pre, code { font-family: Menlo, monospace; font-size: 10pt; }
        pre { background: #f0f0e9; padding: 10px; }
        table { border-collapse: collapse; width: 100%; } th, td { border-bottom: 1px solid #d9d7ce; padding: 6px; }
        img { max-width: 100%; }
      </style></head><body>${editor.innerHTML}</body></html>`;
    },
    preparePrint: () => {
      if (document.body.classList.contains('source-mode')) render(source.value);
    }
  };
  setMarkdown('', 'Untitled.md', 'file:///');
  webkit.messageHandlers.ready.postMessage(true);
})();
