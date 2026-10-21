// Property handles in document coordinates. The canvas owns events, drawing and
// history; this module owns the geometry and returns patches without mutating.
import { HANDLES, handlePoints, unionBox } from './geometry.js';
import { appliedRotation, canRotate, centerOf, normalizeDegrees, rotatePoint, toLocal } from './rotation.js';
import { cornerValue, radiiOf, withCorner, withRadius } from '../corner-radii.js';
import { fitGauge, lookOf, share } from '../needle-look.js';
import { spectrumOf } from '../spectrum.js';
import { pathPatch } from '../inspector-edit.js';

const clamp = (n, min, max) => Math.max(min, Math.min(max, n));
const degrees = (p, center) => Math.atan2(p.x - center.x, center.y - p.y) * 180 / Math.PI;
const atAngle = (center, radius, angle) => ({ x: center.x + radius * Math.sin(angle * Math.PI / 180), y: center.y - radius * Math.cos(angle * Math.PI / 180) });
const distance = (p, q) => Math.hypot(p.x - q.x, p.y - q.y);
const nearestAngle = (angle, reference) => reference + normalizeDegrees(angle - reference);

/** Eight common square handles; rotation is offered only for eligible objects. */
export function frameHandles(elements, zoom = 1) {
  if (!elements.length || elements.some(e => e.locked)) return [];
  const single = elements.length === 1;
  const frame = single ? elements[0].frame : unionBox(elements.map(e => e.frame));
  const angle = single ? appliedRotation(elements[0]) : 0, center = centerOf(frame);
  const result = HANDLES.map(id => {
    const [x, y] = handlePoints(frame)[id];
    return { id, symbol: 'size', ...rotatePoint({ x, y }, center, angle) };
  });
  if (elements.every(e => canRotate(null, e))) result.push({ id: 'rotation', symbol: 'rotation', ...rotatePoint({ x: center.x, y: frame.y - 24 / zoom }, center, angle) });
  return result;
}

/** Scale root frames proportionally; container descendants follow setFrame. */
export function scaledFrames(elements, from, to) {
  if (!(from.width > 0 && from.height > 0 && to.width >= 4 && to.height >= 4)) return [];
  return elements.map(e => ({ id: e.id, patch: { frame: {
    x: to.x + (e.frame.x - from.x) * to.width / from.width,
    y: to.y + (e.frame.y - from.y) * to.height / from.height,
    width: Math.max(4, e.frame.width * to.width / from.width),
    height: Math.max(4, e.frame.height * to.height / from.height),
  } } }));
}

/** Handles for properties already supported by the theme format. */
export function propertyHandles(element, { fraction = .5, coverFrame = null } = {}) {
  if (element.locked || element.isGroup) return [];
  const f = element.frame, k = element.kind, center = centerOf(f), angle = appliedRotation(element);
  const result = [];
  const add = (id, symbol, field, unit, value, point, edit) => result.push({ id, symbol, field, unit, value, ...rotatePoint(point, center, angle), edit });
  const property = (id, symbol, field, unit, value, point, path, read, present = next => next) => add(id, symbol, field, unit, value, point, (p, options) => {
    const next = read(p, options); return { patch: pathPatch(path, next), value: present(next) };
  });
  const arc = (pivot, radius, direction = 1) => {
    const start = k.startAngle, sweep = k.sweep, end = start + direction * sweep;
    const readAngle = (p, shift, reference) => {
      const value = nearestAngle(degrees(p, pivot), reference);
      return shift ? Math.round(value / 15) * 15 : Math.round(value);
    };
    add('arc-start', 'point', 'inspector.startAngle', '°', start, atAngle(pivot, radius, start), (p, { shift = false }) => {
      const value = clamp(readAngle(p, shift, start), -360, 360);
      return { value, patch: { kind: { startAngle: value, sweep: clamp((end - value) / direction, 0, 360) } } };
    });
    property('arc-end', 'point', 'inspector.sweep', '°', sweep, atAngle(pivot, radius, end), ['kind', 'sweep'], (p, { shift = false }) => clamp((readAngle(p, shift, end) - start) / direction, 0, 360));
  };
  if (k.type === 'ring') {
    const outer = Math.min(f.width, f.height) / 2, thickness = Math.min(k.thickness, outer);
    arc(center, outer - thickness / 2, k.clockwise === false ? -1 : 1);
    property('thickness', 'measure', 'insp.thickness', 'px', thickness, { x: center.x, y: center.y - outer + thickness }, ['kind', 'thickness'], p => clamp(Math.round(outer - distance(p, center)), 1, outer));
  }
  if (k.type === 'needle') {
    const look = lookOf(k.look), legacy = !k.look;
    const pivotValue = k.pivot ?? [.5, .5];
    const gauge = legacy ? { pivot: [f.x + f.width * pivotValue[0], f.y + f.height * pivotValue[1]], radius: Math.min(f.width, f.height) * .45 } : fitGauge(f, pivotValue, k.startAngle, k.sweep, look.arcWidth / 2 + 1);
    const pivot = { x: gauge.pivot[0], y: gauge.pivot[1] };
    arc(pivot, gauge.radius);
    add('pivot', 'point', 'inspector.pivotX', '%', pivotValue.map(v => v * 100), pivot, p => {
      const value = [clamp((p.x - f.x) / f.width, 0, 1), clamp((p.y - f.y) / f.height, 0, 1)];
      return { patch: { kind: { pivot: value } }, value: value.map(v => v * 100) };
    });
    property('needle-length', 'measure', 'insp.needle.length', '%', look.length, atAngle(pivot, gauge.radius * look.length / 100, k.startAngle + k.sweep * fraction), ['kind', 'look', 'length'], p => clamp(Math.round(distance(p, pivot) / Math.max(gauge.radius, 1) * 100), 10, 150));
    look.zones.slice(0, -1).forEach((zone, index) => {
      const value = zone.to;
      add(`zone-${index}`, 'point', 'insp.needle.zoneTo', '', value, atAngle(pivot, gauge.radius, k.startAngle + k.sweep * share(k.binding, value)), p => {
        const position = k.sweep === 0 ? share(k.binding, value) : clamp((nearestAngle(degrees(p, pivot), k.startAngle + k.sweep * share(k.binding, value)) - k.startAngle) / k.sweep, 0, 1);
        const next = clamp(k.binding.min + position * (k.binding.max - k.binding.min), zone.from, look.zones[index + 1].to);
        const zones = look.zones.map((z, i) => i === index ? { ...z, to: next } : i === index + 1 ? { ...z, from: next } : { ...z });
        return { patch: { kind: { look: { zones } } }, value: next };
      });
      result.at(-1).fieldParams = { number: index + 1 };
    });
  }
  if ((k.type === 'shape' && k.shape === 'rect') || k.type === 'bar' || k.type === 'image') {
    const corners = cornerValue(k.radius ?? 0, k.corners);
    radiiOf(corners).forEach((radius, index) => {
      const right = index === 1 || index === 2, bottom = index >= 2;
      // Separate a small corner control from the square resize handle. Its
      // offset is subtracted during projection, so the first drag never jumps.
      const inset = Math.max(radius, 12), offset = inset - radius;
      const point = { x: f.x + (right ? f.width - inset : inset), y: f.y + (bottom ? f.height - inset : inset) };
      add(`corner-${index}`, 'point', 'inspector.radius', 'px', radius, point, (p, { alt = false }) => {
        const value = clamp(Math.round(((right ? f.x + f.width - p.x : p.x - f.x) + (bottom ? f.y + f.height - p.y : p.y - f.y)) / 2 - offset), 0, Math.min(f.width, f.height) / 2);
        return { patch: { kind: alt ? withCorner(corners, index, value) : withRadius(corners, value) }, value };
      });
    });
  }
  if (k.type === 'bar') {
    const vertical = ['bottomToTop', 'topToBottom'].includes(k.direction), reverse = ['bottomToTop', 'rightToLeft'].includes(k.direction);
    const value = reverse ? 1 - fraction : fraction;
    add('trial-value', 'measure', 'insp.tryObject', '%', fraction * 100, { x: vertical ? center.x : f.x + f.width * value, y: vertical ? f.y + f.height * value : center.y }, p => {
      const at = clamp(vertical ? (p.y - f.y) / f.height : (p.x - f.x) / f.width, 0, 1);
      return { trial: reverse ? 1 - at : at, value: (reverse ? 1 - at : at) * 100 };
    });
  }
  if (k.type === 'graph') {
    ['max', 'min'].forEach((key, i) => property(`scale-${key}`, 'point', `inspector.${key}`, '', k.binding[key], { x: f.x + f.width, y: f.y + f.height * i }, ['kind'], p => {
      const value = k.binding[key] + (f.y + f.height * i - p.y) / f.height * (k.binding.max - k.binding.min);
      return { autoscale: false, binding: { [key]: key === 'min' ? Math.min(value, k.binding.max - .001) : Math.max(value, k.binding.min + .001) } };
    }, value => value.binding[key]));
  }
  if (k.type === 'text' && ['static', 'sensor', 'clock'].includes(k.content.type) && !k.content.analog) {
    property('text-size', 'measure', 'inspector.size', 'px', k.style.size, { x: f.x + f.width + 12, y: f.y + f.height + 12 }, ['kind', 'style', 'size'], p => clamp(Math.round(k.style.size * distance(p, { x: f.x, y: f.y }) / Math.max(1, Math.hypot(f.width + 12, f.height + 12))), 4, 800));
  }
  if (k.type === 'text' && k.content.type === 'spectrum') {
    const s = spectrumOf(k.content);
    property('bands', 'measure', 'spectrum.bands', '', s.bands, { x: f.x + f.width, y: center.y }, ['kind', 'content', 'bands'], p => clamp(Math.round(s.bands + (p.x - f.x - f.width) / Math.max(1, s.thickness + s.gap)), 8, 64));
    property('bar-gap', 'point', 'spectrum.gap', 'px', s.gap, { x: f.x + s.thickness + s.gap, y: f.y + f.height }, ['kind', 'content', 'gap'], p => clamp(p.x - f.x - s.thickness, 0, 128));
  }
  const cover = k.type === 'text' && k.content.type === 'player';
  const fade = cover ? k.content.coverFade : k.fade;
  const fadeFrame = cover ? coverFrame : f;
  if (fade && fadeFrame) {
    const path = cover ? ['kind', 'content', 'coverFade'] : ['kind', 'fade'];
    const ff = fadeFrame, fc = centerOf(ff);
    const origin = fade.radial ? { x: ff.x + fade.radial[0] * ff.width, y: ff.y + fade.radial[1] * ff.height } : fc;
    if (fade.radial) {
      property('fade-center', 'point', 'fade.centerX', '%', fade.radial.slice(0, 2).map(v => v * 100), origin, path, p => ({ radial: [clamp((p.x - ff.x) / ff.width, 0, 1), clamp((p.y - ff.y) / ff.height, 0, 1), fade.radial[2]] }), next => next.radial.slice(0, 2).map(v => v * 100));
      property('fade-radius', 'measure', 'fade.extent', '%', fade.radial[2] * 100, { x: origin.x + fade.radial[2] * Math.max(ff.width, ff.height), y: origin.y }, path, p => ({ radial: [fade.radial[0], fade.radial[1], clamp(distance(p, origin) / Math.max(ff.width, ff.height), .01, 2)] }), next => next.radial[2] * 100);
    } else {
      const reach = Math.min(ff.width, ff.height) * (fade.midpoint ?? .5);
      property('fade-direction', 'measure', 'shape.fadeAngle', '°', fade.angle, atAngle(fc, reach, fade.angle + 90), path, p => ({ angle: normalizeDegrees(degrees(p, fc) - 90), midpoint: clamp(distance(p, fc) / Math.min(ff.width, ff.height), .05, .95) }), next => next.angle);
    }
  }
  return result;
}

/** A pointer is inverse-rotated once; invalid input and stale handles do nothing. */
export function editPropertyHandle(element, id, point, options = {}) {
  if (![point.x, point.y].every(Number.isFinite)) return null;
  const handle = propertyHandles(element, options).find(handle => handle.id === id);
  if (!handle) return null;
  return { field: handle.field, fieldParams: handle.fieldParams, unit: handle.unit, ...handle.edit(toLocal(element.frame, appliedRotation(element), point), options) };
}

/** A scale endpoint's double click restores the graph's native auto scale. */
export function resetPropertyHandle(element, id) {
  return !element.locked && element.kind.type === 'graph' && ['scale-min', 'scale-max'].includes(id)
    ? { kind: { autoscale: true } } : null;
}
