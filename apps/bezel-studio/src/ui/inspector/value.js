import { describeSections } from './sections.js';
import { textStyleFields, overflowField, shadowSection, valueFormatFields } from './text.js';
// Existing value controls, isolated from inspector navigation.
import { el } from '../dom.js';
import { propertySection as section, iconChoice, chipButton, textField } from '../fields.js';
import { sensorPicker } from '../sensor-picker.js';

export function panels(ctx) {
  const { t, update, sensors, sourceSection, fontFields, transitionSection } = ctx;
  function textForm(e) {
    const k = e.kind;
    const c = k.content;
    const nodes = ctx.extra?.nodes ?? [], weatherAppearance = ctx.extra?.weatherAppearance ?? [];
    const setContent = content => ctx.store.dispatch('setKind', { id: e.id, kind: { ...k, content } });
    if (['static', 'sensor'].includes(c.type)) {
      nodes.push(iconChoice(t('inspector.content'), c.type, [['static', t('widget.text')], ['sensor', t('inspector.sensor')]], type => {
        if (type === c.type) return;
        if (type === 'static') setContent({ type: 'static', text: ctx.sampleText(e) });
        else {
          const sensor = sensors.catalog()[0];
          if (sensor) setContent({ type: 'sensor', key: sensor.key, prefix: '', suffix: '', format: { showUnit: true } });
        }
      }, { wide: true }));
    }
    if (c.type === 'static') {
      nodes.push(textField(t('common.text'), c.text, text => update(e.id, { kind: { content: { text } } }), { multiline: true }));
      const hostname = sensors.catalog().find(sensor => sensor.key === 'system.hostname');
      nodes.push(el('div', { class: 'chip-group', role: 'group', 'aria-label': t('insp.insertLive') }, [
        chipButton(t('insp.insertTime'), false, () => setContent({ type: 'clock', pattern: '%H:%M' })),
        chipButton(t('insp.insertDate'), false, () => setContent({ type: 'clock', pattern: '%d/%m/%Y' })),
        hostname && chipButton(t('insp.insertHostname'), false, () => setContent({ type: 'sensor', key: hostname.key, prefix: '', suffix: '', format: { showUnit: false } })),
      ]));
      nodes.push(sensorPicker({ t, key: '', catalog: () => sensors.catalog(), readings: () => ctx.host.lastSample()?.readings, label: t('insp.insertSensor'),
        onChange: key => setContent({ type: 'sensor', key, prefix: '', suffix: '', format: { showUnit: true } }) }));
    }


    const data = [];
    if (c.type === 'sensor') {
      data.push(sourceSection(e, c.key, (key) => ({ kind: { content: { key } } })));
      nodes.push(...valueFormatFields(ctx, e, c, content => update(e.id, { kind: { content } })));
    }
    data.push(section('content', t('inspector.content'), nodes));
    if (c.type === 'sensor') data.push(transitionSection(e));
    const look = [
      section('appearance', t('inspector.appearance'), weatherAppearance),
      section('font', t('inspector.font'), ['static', 'sensor'].includes(c.type) ? textStyleFields(ctx, e, { tabular: c.type === 'sensor' }) : fontFields(e)),
    ];
    if (['static', 'sensor'].includes(c.type)) {
      data.push(section('overflow', t('insp.textOverflow'), [overflowField(ctx, e)]));
      look.push(shadowSection(ctx, e));
    }
    return { data, look };
  }
  return textForm(ctx.element, ctx.assets);
}

export function sections(ctx) {
  return describeSections(ctx, ctx.completePanels(panels(ctx)));
}
