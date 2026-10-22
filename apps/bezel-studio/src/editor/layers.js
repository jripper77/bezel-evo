import { withChildren, selectionRoots, owners } from './cards.js';
// Presentation only: persistent paint order remains a flat array.
export function layerGroups(elements) {
  const nodes = new Map(elements.map(element => [element.id, { element, groups: element.card ?
    [null, ...element.card.faces.map((_, i) => i)].map(face => ({ face, elements: [], nodes: [] })) : [], children: [] }]));
  const roots = [];
  for (const e of [...elements].reverse()) {
    const node = nodes.get(e.id), group = nodes.get(e.groupParent);
    if (group?.element.isGroup && group !== node) { group.children.push(node); continue; }
    const card = nodes.get(e.cardMember?.parent);
    const face = card?.groups.find(g => g.face === e.cardMember.face);
    if (!e.card && face) { face.elements.push(e); face.nodes.push(node); }
    else roots.push(node);
  }
  return roots;
}

/** Drop above/below a displayed sibling; return the post-removal paint index. */
export function layerDropIndex(elements, id, targetId, before) {
  if (id === targetId) return null;
  const roots = layerGroups(elements);
  const lists = [];
  const visit = nodes => {
    lists.push(nodes.map(n => n.element));
    for (const node of nodes) {
      if (node.children.length) visit(node.children);
      for (const face of node.groups) visit(face.nodes);
    }
  };
  visit(roots);
  const siblings = lists.find(group => group.some(e => e.id === id));
  if (!siblings?.some(e => e.id === targetId)) return null;
  const theme = { elements };
  const moved = new Set(withChildren(theme, [id]));
  const anchors = new Set(withChildren(theme, [targetId]));
  const remaining = elements.filter(e => !moved.has(e.id));
  const indices = remaining.flatMap((e, i) => anchors.has(e.id) ? [i] : []);
  if (!indices.length) return null;
  // The list shows topmost first, opposite to the theme's paint order.
  return before ? Math.max(...indices) + 1 : Math.min(...indices);
}

/** Validate a layer reparent/reorder without changing geometry or IDs. */
export function layerPlacement(theme,{ids,id,target,inside=false,face,root=false,before=true}) {
  const roots=theme.elements.filter(e=>(ids ?? [id]).includes(e.id) && !withChildren(theme,(ids ?? [id]).filter(n=>n!==e.id)).includes(e.id));
  if(!roots.length)return null;
  const moved=new Set(withChildren(theme,roots.map(e=>e.id)));
  const owner=theme.elements.find(e=>e.id===target);
  if(!root && (!owner || moved.has(target)))return null;
  let groupParent=null,cardMember=null;
  if(!root) {
    if(inside && owner.isGroup){groupParent=owner.id;cardMember=owner.cardMember ?? null;}
    else if(inside && owner.card){if(face!==null && face!==undefined && (!Number.isInteger(face)||face<0||face>=owner.card.faces.length))return null;cardMember={parent:owner.id,face:face===undefined?owner.card.activeFace:face};}
    else if(inside)return null;
    else {groupParent=owner.groupParent ?? null;cardMember=owner.cardMember ?? null;}
  }
  const locked=e=>e?.locked || (e && (e.groupParent!=null || e.cardMember) && theme.elements.some(p=>(p.id===e.groupParent || p.id===e.cardMember?.parent)&&locked(p)));
  const changed=roots.some(e=>(e.groupParent ?? null)!==groupParent || (e.cardMember?.parent ?? null)!==(cardMember?.parent ?? null) || (e.cardMember?.face ?? null)!==(cardMember?.face ?? null));
  const container=theme.elements.find(e=>e.id===(groupParent ?? cardMember?.parent));
  if(changed && (roots.some(locked) || locked(container)))return null;
  if(cardMember && theme.elements.some(e=>moved.has(e.id)&&e.card))return null;
  return {roots:roots.map(e=>e.id),moved,owner,groupParent,cardMember,inside,root,before,changed};
}


/** Reorder sibling roots as blocks, retaining face/group membership and locks.
 * `step` crosses one unselected sibling; `front` moves to the scope's edge.
 */
export function orderLayers(theme, ids, { step, front } = {}) {
  if (step !== 1 && step !== -1 && typeof front !== 'boolean') return theme;
  const selected = new Set(selectionRoots(theme, ids).filter(e => !owners(theme, e).some(p => p.locked)).map(e => e.id));
  if (!selected.size) return theme;
  const lists = [];
  const visit = nodes => {
    lists.push(nodes.map(n => n.element.id).reverse());
    for (const n of nodes) { visit(n.children); for (const face of n.groups) visit(face.nodes); }
  };
  visit(layerGroups(theme.elements));
  let next = theme;
  const move = (id, target, above) => {
    const index = layerDropIndex(next.elements, id, target, above);
    if (index === null) return;
    const moved = new Set(withChildren(next, [id]));
    const block = next.elements.filter(e => moved.has(e.id));
    const elements = next.elements.filter(e => !moved.has(e.id));
    elements.splice(index, 0, ...block);
    if (elements.some((e, i) => e.id !== next.elements[i].id)) next = { ...next, elements };
  };
  for (const list of lists) {
    if (typeof front === 'boolean') {
      const roots = list.filter(id => selected.has(id));
      for (const id of front ? roots : roots.reverse()) {
        const target = front ? list.at(-1) : list[0];
        if (id === target) continue;
        move(id, target, front);
        list.splice(list.indexOf(id), 1);
        if (front) list.push(id); else list.unshift(id);
      }
    } else {
      for (let i = step > 0 ? list.length - 2 : 1; step > 0 ? i >= 0 : i < list.length; i -= step) {
        const j = i + step;
        if (!selected.has(list[i]) || selected.has(list[j])) continue;
        move(list[i], list[j], step > 0);
        [list[i], list[j]] = [list[j], list[i]];
      }
    }
  }
  return next.elements.every((e, i) => e.id === theme.elements[i].id) ? theme : next;
}

/** The UI and store share exactly the same boundary and lock rules. */
export function layerOrderAvailability(theme, ids) {
  return Object.fromEntries(Object.entries({ front: { front: true }, forward: { step: 1 }, backward: { step: -1 }, back: { front: false } })
    .map(([key, options]) => [key, orderLayers(theme, ids, options) !== theme]));
}
