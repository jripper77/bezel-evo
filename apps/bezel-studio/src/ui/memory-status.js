// The statusbar's Bezel chip (D-2026-10-20-studio-ux-v2-11, board
// StatusBarNew): the last render time and the memory, and its dialog with
// the render times of the last minute, the memory of each process and Copy
// diagnostics. Native popover behavior provides click-outside, Escape and
// keyboard focus return without another copy of the dialog's event handling.
import { el, icon, svgEl } from './dom.js';
import { ICONS } from './icons.js';
import { iconButton, panelHead, pill } from './fields.js';
import { bezelStatus, createRenderHistory } from '../memory-status.js';
import { sparkPath } from '../library-model.js';

const SPARK = Object.freeze({ width: 280, height: 40 });

export function createMemoryStatus({ t, root, notify = () => {}, now = () => Date.now() }) {
  const ms = el('span', { class: 'status-figure', id: 'status-render' });
  const mb = el('span', { class: 'status-figure status-figure-wide' });
  const name = el('span');
  const toggle = pill('', { as: 'button', dot: true, attrs: { id: 'status-memory-toggle', popovertarget: 'memory-popover', 'aria-haspopup': 'dialog', 'aria-controls': 'memory-popover', 'aria-expanded': 'false' } });
  toggle.append(icon(ICONS.performance, 13), name, ms, el('span', { 'aria-hidden': 'true', text: '·' }), mb);
  const popover = el('div', { id: 'memory-popover', class: 'popover memory-popover status-popover', popover: 'auto', role: 'dialog', 'aria-labelledby': 'memory-title' });
  const close = iconButton(t('dialog.close'), ICONS.close, () => popover.hidePopover(), { class: 'quiet', autofocus: true });
  const head = panelHead(t('shell.bezelTitle'), { id: 'memory-title', tools: [close] });
  const caption = el('p', { class: 'hint' });
  const spark = svgEl('svg', { class: 'render-spark', viewBox: `0 0 ${SPARK.width} ${SPARK.height}`, preserveAspectRatio: 'none', 'aria-hidden': 'true' });
  const line = svgEl('path', {});
  spark.append(line);
  const renderRow = el('p', { class: 'status-row' });
  const list = el('ul', { class: 'memory-processes' });
  const total = el('p', { class: 'status-row status-total' });
  const note = el('p', { class: 'hint' });
  const copy = el('button', { type: 'button', class: 'text-button', onclick: () => copyDiagnostics() });
  popover.append(head, caption, spark, renderRow, list, total, note, el('div', { class: 'status-actions' }, [copy]));
  root.append(toggle);
  document.body.append(popover);
  const history = createRenderHistory();
  let last = null;
  // A failed render, said until the next frame renders.
  let error = null;

  const status = () => bezelStatus(history.last(), history.values(), last, t);

  function render() {
    const s = status();
    name.textContent = t('shell.bezel');
    ms.textContent = history.last() === null ? '' : t('shell.ms', { ms: Math.round(history.last()) });
    mb.textContent = s.memory.amount;
    toggle.dataset.tone = error ? 'warn' : s.tone;
    toggle.title = [s.label, error, s.memory.note].filter(Boolean).join('\n');
    toggle.dataset.demo = String(s.memory.demo);
    head.querySelector('h2').textContent = t('shell.bezelTitle');
    close.title = t('dialog.close');
    close.setAttribute('aria-label', t('dialog.close'));
    caption.textContent = t('shell.bezelCaption');
    const d = sparkPath(history.values(), SPARK.width, SPARK.height);
    if (d) line.setAttribute('d', d); else line.removeAttribute('d');
    renderRow.replaceChildren(el('span', { text: t('shell.bezelRenderLabel') }), el('strong', { class: 'mono', text: error ?? s.render }));
    list.replaceChildren(...s.memory.processes.map(p => el('li', { title: t('shell.memoryProcess', p) }, [el('span', { text: p.name }), el('span', { class: 'mono', text: p.value })])));
    total.replaceChildren(el('span', { text: t('shell.bezelTotal') }), el('strong', { class: 'mono', text: s.memory.amount }));
    note.textContent = s.memory.note;
    copy.textContent = t('shell.copyDiagnostics');
  }

  async function copyDiagnostics() {
    try {
      await navigator.clipboard.writeText(status().diagnostics);
      notify(t('shell.diagnosticsCopied'));
    } catch {
      notify(t('shell.diagnosticsNotCopied'));
    }
  }

  function place() {
    const rect = toggle.getBoundingClientRect();
    popover.style.right = `${Math.max(8, window.innerWidth - rect.right)}px`;
    popover.style.bottom = `${window.innerHeight - rect.top + 8}px`;
  }
  popover.addEventListener('beforetoggle', evt => {
    toggle.setAttribute('aria-expanded', String(evt.newState === 'open'));
    if (evt.newState === 'open') place();
  });
  window.addEventListener('resize', () => { if (popover.matches(':popover-open')) place(); });
  render();
  return {
    update(report) { last = report; render(); },
    /** A frame of the canvas took `millis` to render. */
    rendered(millis) { error = null; history.push(millis, now()); render(); },
    /** The canvas could not render: `message` says why. */
    failed(message) { error = message; render(); },
    retranslate: render,
  };
}
