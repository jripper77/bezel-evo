// Browser demo of the same optional geometries as the native renderer.
export function drawVisualization(ctx, e, fraction, points, secondaryPoints) {
  const v=e.visualization, k=e.kind, f=e.frame;
  if (!v) return false;
  const box=(x,y,w,h,r,fill)=>{ if (!fill || w<=0 || h<=0) return; ctx.fillStyle=fill;ctx.beginPath();ctx.roundRect(x,y,w,h,Math.max(0,Math.min(r,w/2,h/2)));ctx.fill(); };
  const paint=(p,position=0)=>{
    if (typeof p==='string') return p;
    if(p?.scale==='arc') {
      const mix=(a,b)=>Math.round(parseInt(a,16)*(1-position)+parseInt(b,16)*position).toString(16).padStart(2,'0');
      return '#'+[1,3,5,7].map(i=>mix(p.start.slice(i,i+2),p.end.slice(i,i+2))).join('');
    }
    if(p?.stops?.length) {const g=ctx.createLinearGradient(f.x,f.y,f.x+f.width,f.y+f.height);p.stops.forEach(([at,c])=>g.addColorStop(at,c));return g;}
    return null;
  };
  if(k.type==='bar' && v.style==='matrix') {
    const cols=Math.max(1,Math.min(64,v.columns)),rows=Math.max(1,Math.min(32,v.rows));
    const w=f.width/cols,h=f.height/rows,gap=Math.min(v.gap,w*.8,h*.8),lit=Math.floor(fraction*cols*rows);
    for(let i=0;i<cols*rows;i++) {
      let x=i%cols,y=rows-1-Math.floor(i/cols);
      if(k.direction==='rightToLeft')x=cols-1-x;
      if(['topToBottom','bottomToTop'].includes(k.direction)){x=Math.floor(i/rows);y=k.direction==='topToBottom'?i%rows:rows-1-i%rows;}
      box(f.x+x*w+gap/2,f.y+y*h+gap/2,w-gap,h-gap,v.radius,paint(i<lit?k.fill:k.track));
    }
    return true;
  }
  if(k.type==='ring' && ['pixel','dial'].includes(v.style)) {
    const cx=f.x+f.width/2,cy=f.y+f.height/2,outer=Math.min(f.width,f.height)/2;
    const start=k.startAngle,sweep=Math.min(360,k.sweep),dir=k.clockwise===false?-1:1,value=k.testFull?1:fraction;
    const point=(r,p)=>{const a=(start+dir*sweep*p-90)*Math.PI/180;return [cx+r*Math.cos(a),cy+r*Math.sin(a)];};
    if(v.style==='pixel') {
      const side=Math.max(1,Math.min(k.thickness,outer/2)),r=outer-side*1.42/2,count=Math.max(2,Math.min(120,v.ticks)),lit=Math.floor(value*count);
      for(let i=0;i<count;i++){const p=i/(sweep>=360?count:count-1),[x,y]=point(r,p);box(x-side/2,y-side/2,side,side,v.radius,paint(i<lit?k.fill:k.track,p));}
    } else {
      const r=outer*.76-k.thickness/2;
      ctx.lineWidth=k.thickness;ctx.lineCap=k.roundCaps?'round':'butt';
      const arc=(a,b,p)=>{if(!p || b<=a || r<=0)return;ctx.strokeStyle=p;ctx.beginPath();ctx.arc(cx,cy,r,(start+dir*sweep*a-90)*Math.PI/180,(start+dir*sweep*b-90)*Math.PI/180,dir<0);ctx.stroke();};
      arc(0,1,paint(k.track));
      if(k.fill?.scale==='arc'){for(let i=0;i<100;i++)arc(i/100,Math.min(value,(i+1)/100),paint(k.fill,i/100));}else arc(0,value,paint(k.fill));
      ctx.lineWidth=1.5;ctx.lineCap='butt';ctx.strokeStyle=paint(k.track)||paint(k.fill);
      for(let i=0;i<v.ticks;i++){const p=i/(v.ticks-1),a=point(outer*.9,p),b=point(outer*(i%5===0?.98:.95),p);ctx.beginPath();ctx.moveTo(...a);ctx.lineTo(...b);ctx.stroke();}
      if(v.marker){const [x,y]=point(r,value);ctx.strokeStyle=paint(k.fill,value);ctx.lineWidth=2.5;ctx.beginPath();ctx.arc(x,y,Math.max(k.thickness*.6,v.radius),0,Math.PI*2);if(k.track){ctx.fillStyle=paint(k.track);ctx.fill();}ctx.stroke();}
    }
    return true;
  }
  if(k.type==='graph' && ['rounded','mirrored'].includes(v.style)) {
    const graph=(values,area,reverse,color)=>{
      if(!values?.length)return;
      ctx.lineWidth=k.lineWidth;ctx.strokeStyle=color;ctx.fillStyle=color;ctx.lineJoin='round';
      const base=reverse?area.y:area.y+area.height;
      const coords=values.map((value,i)=>[area.x+area.width*i/(values.length-1),base+(reverse?1:-1)*value*area.height]);
      if(k.style==='bars') {
        const pitch=area.width/values.length,gap=Math.min(v.gap,pitch*.8);
        const radius = v.roundedBars !== false ? (pitch-gap)/2 : 0;
        values.forEach((value,i)=>box(area.x+i*pitch+gap/2,reverse?base:base-value*area.height,pitch-gap,value*area.height,radius,color));
      } else {
        if(k.style==='area'){ctx.globalAlpha*=.28;ctx.beginPath();ctx.moveTo(area.x,base);coords.forEach(([x,y])=>ctx.lineTo(x,y));ctx.lineTo(area.x+area.width,base);ctx.closePath();ctx.fill();ctx.globalAlpha/=.28;}
        ctx.beginPath();coords.forEach(([x,y],i)=>i?ctx.lineTo(x,y):ctx.moveTo(x,y));ctx.stroke();
      }
    };
    if(v.style==='mirrored'){
      const gap=v.seriesGap ?? 2,h=Math.max(0,(f.height-gap)/2);graph(points,{...f,height:h},false,k.color);
      if(v.secondary)graph(secondaryPoints,{...f,y:f.y+h+gap,height:h},true,v.secondaryColor);
    }else graph(points,f,false,k.color);
    return true;
  }
  return false;
}
