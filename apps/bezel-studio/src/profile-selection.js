// Demo counterpart of runtime::profiles::Selection. Seconds are monotonic;
// local time is supplied separately so midnight and weekdays are testable.
import { minuteOf } from './profiles-model.js';
import { triggerMatches } from './card-rules.js';

export function profileRuleAvailable(rule, sample, time) {
  switch (rule.source) {
    case 'foreground': case 'process': return sample.activity?.available === true;
    case 'mediaPlaying': return (sample.activity?.mediaAvailable ?? sample.activity?.available) === true;
    case 'sensor': return Number.isFinite(sample.readings?.[rule.key]?.value);
    case 'time': return time != null;
    case 'idle': return Number.isFinite(sample.idleSeconds);
    case 'locked': return typeof sample.locked === 'boolean';
    default: return false;
  }
}

export function profileRuleHolds(rule, sample, time) {
  if (!profileRuleAvailable(rule, sample, time)) return false;
  switch (rule.source) {
    case 'foreground': case 'process': case 'mediaPlaying': return triggerMatches(rule, sample.activity);
    case 'sensor': return rule.compare === 'above' ? sample.readings[rule.key].value > rule.value
      : rule.compare === 'below' && sample.readings[rule.key].value < rule.value;
    case 'idle': return sample.idleSeconds >= rule.minutes * 60;
    case 'locked': return sample.locked;
    case 'time': {
      const from = minuteOf(rule.from), to = minuteOf(rule.to);
      if (from === null || to === null) return false;
      const minute = time.hour * 60 + time.minute;
      const overnight = from > to;
      const day = overnight && minute < from ? (time.weekday + 6) % 7 : time.weekday;
      return (overnight ? minute >= from || minute < to : minute >= from && minute < to)
        && (!rule.days?.length || rule.days.includes(day));
    }
    default: return false;
  }
}

export function createProfileSelection() {
  let signature, since = new Map(), active = null, release = null, priority = 0, back = 0, reason = [];
  return {
    evaluate(config, sample, now, time) {
      const nextSignature = JSON.stringify(config);
      if (signature !== nextSignature) {
        signature = nextSignature; since = new Map(); active = null; release = null; priority = 0; back = 0; reason = [];
      }
      const nextSince = new Map(), truths = {};
      let best = null;
      for (const profile of config.profiles) {
        const states = profile.rules.map((rule, index) => {
          if (!profileRuleHolds(rule, sample, time)) return false;
          const key = JSON.stringify([profile.id, index]), start = since.get(key) ?? now;
          nextSince.set(key, start);
          return Math.max(0, now - start) >= (rule.forSeconds ?? 0);
        });
        Object.defineProperty(truths, profile.id, { value: states, enumerable: true });
        const candidates = profile.match === 'all'
          ? profile.rules.length && states.every(Boolean) ? [{ priority: Math.max(...profile.rules.map(r => r.priority)), back: Math.max(...profile.rules.map(r => r.returnSeconds)), rules: profile.rules.map((_, i) => i) }] : []
          : profile.rules.flatMap((rule, index) => states[index] ? [{ priority: rule.priority, back: rule.returnSeconds, rules: [index] }] : []);
        for (const candidate of candidates) if (!best || candidate.priority > best.priority) best = { ...candidate, profile: profile.id };
      }
      since = nextSince;
      let matched = false;
      if (!config.automatic) { active = config.manual ?? null; release = null; }
      else if (best) {
        if (active !== null && active !== best.profile && best.priority < priority) release ??= now + back;
        if (!(active !== null && active !== best.profile && best.priority < priority && now < release)) {
          active = best.profile; priority = best.priority; back = best.back; reason = best.rules; matched = true; release = null;
        }
      } else {
        const profile = config.profiles.find(p => p.id === active);
        if (profile) {
          if (!profile.rules.some(rule => !profileRuleAvailable(rule, sample, time))) {
            release ??= now + back;
            if (now >= release) { active = null; release = null; }
          }
        } else { active = null; release = null; }
      }
      const chosen = active ?? (config.automatic && config.profiles.some(p => p.id === config.defaultProfile) ? config.defaultProfile : null);
      return { automatic: config.automatic, ...(chosen === null ? {} : { active: { profile: chosen, reason: !config.automatic ? { kind: 'manual' }
        : active === chosen ? { kind: matched ? 'rules' : 'returning', rules: [...reason] } : { kind: 'default' } } }), rules: truths };
    },
  };
}
