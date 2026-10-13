const { test } = require('node:test');
const assert = require('node:assert/strict');
const path = require('node:path');
const { JSDOM } = require('jsdom');

async function editorWindow(onChanged = () => {}, onCommand = () => {}) {
  const html = path.join(__dirname, '..', 'build', 'MarkdownDesk.app', 'Contents', 'Resources', 'Web', 'index.html');
  const dom = await JSDOM.fromFile(html, {
    resources: 'usable',
    runScripts: 'dangerously',
    beforeParse(window) {
      window.webkit = { messageHandlers: {
        ready: { postMessage() {} },
        changed: { postMessage: onChanged },
        openLink: { postMessage() {} },
        command: { postMessage: onCommand }
      } };
    }
  });
  await new Promise((resolve, reject) => {
    dom.window.addEventListener('load', resolve, { once: true });
    dom.window.addEventListener('error', reject, { once: true });
  });
  return dom;
}

test('New button invokes the native New command', async () => {
  const commands = [];
  const dom = await editorWindow(() => {}, (command) => commands.push(command));
  dom.window.document.querySelector('#new-file').click();
  assert.deepEqual(commands, ['new']);
  dom.window.close();
});

test('opens Markdown as rich text and preserves an unedited file byte for byte', async () => {
  const dom = await editorWindow();
  const text = '---\ntitle: Example\n---\n# Hello\n\nA **bold** [link](https://example.com).\n\n| A | B |\n| - | - |\n| 1 | 2 |\n';
  dom.window.editorAPI.setMarkdown(text, 'Example.md', 'file:///tmp/');
  assert.match(dom.window.document.querySelector('#editor').innerHTML, /<h1>Hello<\/h1>/);
  assert.match(dom.window.document.querySelector('#editor').innerHTML, /<table>/);
  assert.equal(dom.window.editorAPI.getMarkdown(), text);
  dom.window.close();
});

test('rich text changes save as Markdown with front matter and table', async () => {
  const dom = await editorWindow();
  dom.window.editorAPI.setMarkdown('---\ntitle: Example\n---\n# Hello\n\n| A | B |\n| - | - |\n| 1 | 2 |\n', 'Example.md', 'file:///tmp/');
  const paragraph = dom.window.document.createElement('p');
  paragraph.innerHTML = 'A <strong>new</strong> sentence.';
  dom.window.document.querySelector('#editor').appendChild(paragraph);
  dom.window.document.querySelector('#editor').dispatchEvent(new dom.window.Event('input', { bubbles: true }));
  const result = dom.window.editorAPI.getMarkdown();
  assert.match(result, /^---\ntitle: Example\n---\n/);
  assert.match(result, /# Hello/);
  assert.match(result, /\| A \| B \|/);
  assert.match(result, /A \*\*new\*\* sentence\./);
  dom.window.close();
});

test('source edits remain exact and hostile HTML does not execute on render', async () => {
  const dom = await editorWindow();
  dom.window.editorAPI.setMarkdown('', 'Untitled.md', 'file:///tmp/');
  dom.window.editorAPI.toggleSource();
  const source = dom.window.document.querySelector('#source');
  source.value = '# New\n\n<script>window.hacked = true</script>\n';
  source.dispatchEvent(new dom.window.Event('input', { bubbles: true }));
  assert.equal(dom.window.editorAPI.getMarkdown(), source.value);
  dom.window.editorAPI.toggleSource();
  assert.equal(dom.window.hacked, undefined);
  assert.doesNotMatch(dom.window.document.querySelector('#editor').innerHTML, /<script>/);
  dom.window.close();
});

test('the first edit marks the document dirty immediately', async () => {
  let changes = 0;
  const dom = await editorWindow(() => { changes += 1; });
  const editor = dom.window.document.querySelector('#editor');
  editor.textContent = 'New text';
  editor.dispatchEvent(new dom.window.Event('input', { bubbles: true }));
  assert.equal(changes, 1);
  assert.match(dom.window.editorAPI.getMarkdown(), /New text/);
  dom.window.close();
});
