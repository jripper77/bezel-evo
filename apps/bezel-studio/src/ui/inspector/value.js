// Existing value controls, isolated from inspector navigation.
import { el } from '../dom.js';
import { propertySection as section, selectField, switchRow, textField } from '../fields.js';

export function panels(ctx) {
  const { t, update, sensors, sourceSection, fontFields, transitionSection } = ctx;
  function textForm(e) {
    const k = e.kind;
    const s = k.style;
    const c = k.content;
    const nodes = ctx.extra?.nodes ?? [], weatherAppearance = ctx.extra?.weatherAppearance ?? [];
    if (c.type === 'static') nodes.push(textField(t('common.text'), c.text, (v) => update(e.id, { kind: { content: { text: v } } })));


    const data = [];
    if (c.type === 'sensor') {
      data.push(sourceSection(e, c.key, (key) => ({ kind: { content: { key } } })));
      nodes.push(el('div', { class: 'field-row' }, [
        textField(t('inspector.prefix'), c.prefix, (v) => update(e.id, { kind: { content: { prefix: v } } })),
        textField(t('inspector.suffix'), c.suffix, (v) => update(e.id, { kind: { content: { suffix: v } } })),
      ]));
      nodes.push(selectField(t('inspector.decimals'), c.format.decimals === undefined || c.format.decimals === null ? 'auto' : String(c.format.decimals),
        [['auto', t('inspector.decimalsAuto')], ['0', '0'], ['1', '1'], ['2', '2']],
        (v) => update(e.id, { kind: { content: { format: { decimals: v === 'auto' ? null : Number(v) } } } })));
      nodes.push(switchRow(t('inspector.showUnit'), c.format.showUnit !== false, (v) => update(e.id, { kind: { content: { format: { showUnit: v } } } })));
      if (sensors.catalog().find(sensor => sensor.key === c.key)?.quantity === 'celsius') {
        nodes.push(switchRow(t('inspector.fahrenheit'), c.format.fahrenheit, (v) => update(e.id, { kind: { content: { format: { fahrenheit: v } } } })));
      }
    }
    data.push(section('content', t('inspector.content'), nodes));
    if (c.type === 'sensor') data.push(transitionSection(e));
    const look = [
      section('appearance', t('inspector.appearance'), weatherAppearance),
      section('font', t('inspector.font'), fontFields(e)),
    ];
    return { data, look };
  }
  return textForm(ctx.element, ctx.assets);
}
