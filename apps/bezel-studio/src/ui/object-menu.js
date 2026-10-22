import { layerOrderAvailability } from '../editor/layers.js';
import {canFitContent} from '../editor/fit-content.js';
import {SEPARATOR,closeMenu,openMenu,openMenuAt} from './menu.js';
import {canGroup,groupTarget,isShown,owners} from '../editor/cards.js';
import {hitTest} from '../editor/geometry.js';

export function wireObjectMenu({store,canvasView,stage,layers,t,fitContent=()=>{}}) {
 const focusStage=()=>{stage.tabIndex=0;stage.focus({preventScroll:true});};
 function open(evt,keyboard=false){
  if(canvasView.framing() || document.querySelector('dialog[open]'))return;
  const row=evt.target.closest('.layer-row');
  if(!row && !stage.contains(evt.target) && !layers.contains(evt.target))return;
  let {theme,selection}=store.getState();let id=row?Number(row.dataset.elementId):null;
  if(!row && !keyboard){const p=canvasView.point(evt.clientX,evt.clientY);id=hitTest(theme.elements.filter(e=>isShown(theme,e)),p.x,p.y);if(id!==null&&!selection.includes(id))id=groupTarget(theme,id);}
  if(id!==null&&!selection.includes(id)){store.select([id]);selection=[id];}
  if(!selection.length&&!store.canPaste())return;
  evt.preventDefault();
  const items=objectMenuItems({store,t,fitContent});
  const box=keyboard?evt.target.getBoundingClientRect():null;
  openMenu({label:t('menu.title'),items,x:keyboard?box.left:evt.clientX,y:keyboard?box.bottom:evt.clientY,onClose:reason=>{if(reason==='item'||reason==='escape')focusStage();}});
 }
 for(const target of [stage,layers]){target.addEventListener('contextmenu',evt=>open(evt));target.addEventListener('keydown',evt=>{if(evt.key==='ContextMenu'||evt.shiftKey&&evt.key==='F10')open(evt,true);});}
 stage.addEventListener('scroll',()=>closeMenu());
 /** The canvas bar's ⋯: the same items under its button. */
 function openAt(button,onClose=()=>{}){openMenuAt(button,{label:t('menu.title'),items:objectMenuItems({store,t,fitContent}),onClose:reason=>{if(reason==='escape'||reason==='item')onClose();}});}
 return {close:closeMenu,openAt};
}

/** The six alignments, each with its icon (the inspector's row and the layers' Align menu). */
export const ALIGN_EDGES = Object.freeze([['left', 'alignLeft'], ['centerX', 'alignCenterX'], ['right', 'alignRight'], ['top', 'alignTop'], ['centerY', 'alignCenterY'], ['bottom', 'alignBottom']]);

/** Align and distribute `ids`: the menu items (distributing needs three objects). */
export function alignItems({ store, t, ids }) {
  const action = (label, run, disabled = false) => ({ label, run, disabled });
  return [
    ...ALIGN_EDGES.map(([edge]) => action(t(`align.${edge}`), () => store.dispatch('align', { ids, edge }))),
    SEPARATOR,
    action(t('align.distributeX'), () => store.dispatch('distribute', { ids, axis: 'x' }), ids.length < 3),
    action(t('align.distributeY'), () => store.dispatch('distribute', { ids, axis: 'y' }), ids.length < 3),
  ];
}

/** The same actions in the canvas, layers and inspector. */
export function objectMenuItems({store,t,fitContent=()=>{}}) {
  const {theme,selection}=store.getState();
  const selected=theme.elements.filter(e=>selection.includes(e.id));
  const order=layerOrderAvailability(theme,selection);
  const editable=selected.some(e=>!owners(theme,e).some(p=>p.locked));
  const action=(label,run,disabled=false)=>({label,run,disabled});
  const dispatch=(command,args={})=>()=>store.dispatch(command,{ids:selection,...args});
  return [
   action(t('insp.fitContent'),()=>fitContent(selection[0]),selection.length!==1||!canFitContent(theme,selection[0])),
   action(t('menu.cut'),()=>store.cutSelection(),!editable),action(t('menu.copy'),()=>store.copySelection(),!selected.length),
   action(t('menu.paste'),()=>store.paste(),!store.canPaste()),action(t('common.duplicate'),dispatch('duplicate'),!selected.length),
   SEPARATOR,action(t('group.create'),dispatch('groupSelection'),!canGroup(theme,selection)),
   action(t('group.ungroup'),dispatch('ungroup'),!selected.some(e=>e.isGroup&&!e.locked)),
   action(t('card.group'),dispatch('cardFromSelection'),!editable||selected.some(e=>e.card)),
   SEPARATOR,action(t('inspector.front'),dispatch('orderSelection',{front:true}),!order.front),
   action(t('insp.oneForward'),dispatch('stepSelection',{step:1}),!order.forward),
   action(t('insp.oneBackward'),dispatch('stepSelection',{step:-1}),!order.backward),
   action(t('inspector.back'),dispatch('orderSelection',{front:false}),!order.back),
   SEPARATOR,action(t('menu.detach'),dispatch('placeLayer',{root:true}),!selected.some(e=>e.groupParent!=null||e.cardMember)),
   action(t(selected.every(e=>e.locked)?'common.unlock':'common.lock'),dispatch('updateSelection',{patch:{locked:!selected.every(e=>e.locked)}}),!selected.length),
   action(t(selected.every(e=>e.visible===false)?'common.show':'common.hide'),dispatch('updateSelection',{patch:{visible:selected.every(e=>e.visible===false)}}),!selected.length),
   SEPARATOR,action(t('common.delete'),dispatch('remove',{ids:selected.filter(e=>!owners(theme,e).some(p=>p.locked)).map(e=>e.id)}),!editable),
  ];
}
