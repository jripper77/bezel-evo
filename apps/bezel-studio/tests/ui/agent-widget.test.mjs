import { test } from 'node:test';
import assert from 'node:assert/strict';
import { AGENT_COLOR_KEYS, AGENT_QUOTA_COLORS, AGENT_STATE_COLORS, agentColor, quotaValueOf, agentColorReset, agentProjectOptions, agentSessionOptions, projectMatches, agentProjectIncluded, toggleAgentProject } from '../../src/agent-widget.js';
import { createWidget, widgetOf } from '../../src/editor/widgets.js';
import { translator } from '../../src/i18n/index.js';

test('each agent widget has independent configuration', () => {
  const first = createWidget('agent', { width: 480, height: 1920 });
  const second = createWidget('agent', { width: 480, height: 1920 });
  first.kind.content.agent = 'claude';
  first.kind.content.session = 'one';
  first.kind.content.showWeekly = false;
  assert.equal(second.kind.content.agent, 'codex');
  assert.equal(second.kind.content.session, '');
  assert.equal(second.kind.content.showWeekly, true);
  assert.equal(widgetOf(first), 'claude');
});

test('project choices distinguish full paths and sessions stay in their project', () => {
  const t = translator('it');
  const sessions = [
    { agent:'codex', id:'one', project:'bezel', projectPath:'F:\\dev\\bezel' },
    { agent:'codex', id:'two', project:'EIDOLON', projectPath:'F:\\dev\\EIDOLON' },
    { agent:'claude', id:'three', project:'bezel', projectPath:'F:\\dev\\bezel' },
  ];
  const content = { agent:'codex', project:'f:/dev/bezel', session:'' };
  assert.deepEqual(agentSessionOptions(content, sessions, t).map(([id]) => id), ['', 'one']);
  assert.equal(agentProjectOptions(content, sessions, t).length, 3);
  assert.ok(projectMatches('F:/dev/bezel', 'f:/dev/bezel/crates'));
  assert.ok(!projectMatches('F:/dev/bezel', 'F:/dev/bezel-other'));
});

test('session choices filter by agent and retain saved missing sessions', () => {
  const t = translator('it');
  const sessions = [{ agent: 'codex', id: 'one', project: 'bezel' }, { agent: 'claude', id: 'two', project: 'other' }];
  const choices = agentSessionOptions({ agent: 'claude', session: 'saved' }, sessions, t);
  assert.deepEqual(choices.map(([id]) => id), ['', 'two', 'saved']);
  assert.match(choices[1][1], /other/);
  assert.equal(agentSessionOptions({ agent: 'codex' }, null, t).length, 1);
});


test('dedicated providers default to independent automatic project lists', () => {
  const codex = createWidget('codex', { width:1920, height:480 });
  const claude = createWidget('claude', { width:1920, height:480 });
  assert.equal(widgetOf(codex), 'codex');
  assert.equal(widgetOf(claude), 'claude');
  assert.equal(codex.kind.content.projectList, true);
  assert.equal(claude.kind.content.compactQuotas, true);
  codex.kind.content.excludedProjects.push('F:/dev/one');
  assert.deepEqual(claude.kind.content.excludedProjects, []);
});

test('excluded offline projects remain configurable with normalized Windows paths', () => {
  const content = { agent:'codex', excludedProjects:['F:/DEV/one'] };
  assert.equal(agentProjectIncluded(content, 'f:\\dev\\one'), false);
  assert.deepEqual(toggleAgentProject(content, 'f:/dev/one'), []);
  assert.deepEqual(toggleAgentProject(content, 'f:/dev/two'), ['F:/DEV/one', 'f:/dev/two']);
  assert.ok(agentProjectOptions(content, [], translator('it')).some(([path]) => path === 'F:/DEV/one'));
});

test('agent colors fall back to the native palette and reset clears every override', () => {
  assert.equal(agentColor({}, 'workingColor'), '#30bf8bff');
  assert.equal(agentColor({ attentionColor: '#112233ff' }, 'completedColor'), '#112233ff');
  assert.equal(agentColor({ completedColor: '#3060ffff', attentionColor: '#112233ff' }, 'completedColor'), '#3060ffff');
  assert.equal(agentColor({ pixelStyle: true }, 'quotaColor'), '#ff9248ff');
  assert.equal(agentColor({}, 'quotaColor', '#abcdefff'), '#abcdefff');
  assert.equal(agentColor({}, 'quotaColor', { angle: 0, stops: [] }), '#ffffffff');
  assert.equal(agentColor({ pixelStyle: true }, 'quotaTrackColor'), '#222220ff');
  assert.deepEqual(Object.keys(agentColorReset()), [...AGENT_COLOR_KEYS]);
  assert.ok(Object.values(agentColorReset()).every(value => value === null));
});

test('new dedicated widgets show row age and keep the built-in look', () => {
  const content = createWidget('claude', { width: 1920, height: 480 }).kind.content;
  assert.equal(content.showAge, true);
  assert.equal(content.warningPercent, 20);
  assert.ok(content.showHeader && content.showPage && content.showDividers && content.showStateText);
  assert.ok(AGENT_COLOR_KEYS.every(key => content[key] === undefined));
  assert.equal(createWidget('agent', { width: 480, height: 1920 }).kind.content.showAge, false);
});

test('quota value follows the layout until chosen and colors split into two groups', () => {
  assert.equal(quotaValueOf({}), 'both');
  assert.equal(quotaValueOf({ compactQuotas: true }), 'remaining');
  assert.equal(quotaValueOf({ compactQuotas: true, quotaValue: 'used' }), 'used');
  assert.deepEqual([...AGENT_STATE_COLORS, ...AGENT_QUOTA_COLORS], [...AGENT_COLOR_KEYS]);
});

test('parts follow the native layout and the opened part is Studio state', async () => {
  const { agentPartAt, agentPartRects, agentParts, agentPartToggle, agentPartView } = await import('../../src/agent-widget.js');
  const { agentPart, agentPartOwner, focusAgentPart, onAgentPart } = await import('../../src/editor/agent-part.js');
  const element = { id: 7, frame: { x: 0, y: 0, width: 400, height: 300 }, kind: { content: { projectList: true, compactQuotas: true }, style: { size: 20 } } };
  const rects = agentPartRects(element);
  assert.deepEqual(rects.fiveHour, { x: 0, y: 220, width: 400, height: 40 });
  assert.equal(rects.title.height, 33);
  assert.deepEqual([agentPartAt(element, 10, 10), agentPartAt(element, 10, 100), agentPartAt(element, 10, 230), agentPartAt(element, 10, 270)], ['title', 'rows', 'fiveHour', 'weekly']);
  element.kind.style.parts = { agentTitle: { sizePercent: 200 } };
  assert.equal(agentPartRects(element).title.height, 66);
  element.kind.content.showHeader = false;
  assert.equal(agentPartAt(element, 10, 10), 'rows');
  assert.deepEqual(agentParts({ projectList: false }), ['content', 'fiveHour', 'weekly']);
  assert.equal(agentPartView('weekly'), 'quotas');
  assert.deepEqual(agentPartToggle({ showWeekly: false }, 'weekly'), { showWeekly: true });
  assert.equal(agentPartToggle({}, 'rows'), null);
  const seen = [];
  const stop = onAgentPart(focus => seen.push(focus?.part ?? null));
  focusAgentPart(7, 'rows');
  focusAgentPart(7, 'rows');
  assert.equal(agentPart(7), 'rows');
  assert.equal(agentPart(8), null);
  assert.equal(agentPartOwner(), 7);
  focusAgentPart(null, null);
  stop();
  assert.deepEqual(seen, ['rows', null]);
});
