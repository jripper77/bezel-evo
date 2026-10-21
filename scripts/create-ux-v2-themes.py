"""Generate the four editable 1920x480 Studio UX v2 templates. No mock readings."""
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
class Theme:
 def __init__(self,name,bg,ink,muted,font):
  self.ink,self.muted,self.font=ink,muted,font
  self.data=dict(schema=1,name=name,canvas=dict(width=1920,height=480),orientation='landscape',refreshSeconds=1,background=dict(type='color',color=bg),elements=[])
 def add(self,name,box,kind,**extra):
  e=dict(id=len(self.data['elements'])+1,name=name,frame=dict(zip(('x','y','width','height'),box)),kind=kind,**extra);self.data['elements'].append(e);return e
 def text(self,name,box,content,size=16,color=None,font=None,weight=400,align='left',spacing=0,**extra):
  if isinstance(content,str):content=dict(type='static',text=content)
  return self.add(name,box,dict(type='text',content=content,style=dict(font=dict(family=font or self.font,weight=weight),size=size,paint=color or self.ink,align=align,valign='middle',letterSpacing=spacing,overflow='shrink')),**extra)
 def sensor(self,name,box,key,size=20,color=None,font=None,decimals=0,unit=True,**extra):
  e=self.text(name,box,dict(type='sensor',key=key,format=dict(decimals=decimals,showUnit=unit)),size,color,font,**({'weight':900} if font=='Doto' else {}),**extra)
  if size>=32:e['kind']['style']['parts']=dict(unit=dict(font=self.font,sizePercent=40,color=self.muted,weight=400))
  return e
 def clock(self,name,box,pattern,size,color=None,font=None,**extra):
  e=self.text(name,box,dict(type='clock',pattern=pattern),size,color,font,**extra)
  if name in ['Weekday','Date','Edition date'] and size<20 and self.font!='Bricolage Grotesque 96pt':e['kind']['content']['casing']='upper'
  return e
 def shape(self,name,box,fill,radius=0,**extra):
  return self.add(name,box,dict(type='shape',shape='rect',fill=fill,radius=radius,strokeWidth=0),**extra)
 def bar(self,name,box,key,color,track,max=100,**extra):
  segments=extra.pop('segments',None)
  return self.add(name,box,dict(type='bar',binding=dict(key=key,min=0,max=max),direction='leftToRight',fill=color,track=track,radius=2,**({'segments':segments} if segments else {})),**extra)
 def ring(self,name,box,key,fill,track,thickness=12,start=-225,sweep=270,**extra):
  return self.add(name,box,dict(type='ring',binding=dict(key=key,min=0,max=100),fill=fill,track=track,thickness=thickness,startAngle=start,sweep=sweep,clockwise=True,roundCaps=True),**extra)
 def graph(self,name,box,key,color,style='line',max=100,history=60,**extra):
  return self.add(name,box,dict(type='graph',binding=dict(key=key,min=0,max=max),history=history,intervalSeconds=1,style=style,color=color,lineWidth=1.5,autoscale=False,**({'fill':dict(angle=90,stops=[[0,color[:7]+'60'],[1,color[:7]+'00']])} if style=='area' else {})),**extra)
 def svg(self,name,body):
  self.asset=body
  return self.add(name,(0,0,1920,480),dict(type='image',asset='assets/architecture.svg',fit='fill'),locked=True)
 def save(self,slug):
  d=ROOT/'themes'/slug;(d/'assets').mkdir(parents=True,exist_ok=True)
  (d/'theme.json').write_text(json.dumps(self.data,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
  (d/'assets/architecture.svg').write_text('<svg xmlns="http://www.w3.org/2000/svg" width="1920" height="480" viewBox="0 0 1920 480">'+self.asset+'</svg>\n',encoding='utf-8')

def pixel():
 t=Theme('Pixel','#000000ff','#F2EDE6ff','#5E5A53ff','IBM Plex Mono'); orange='#FF9248ff'; grey='#8A857Cff'
 # Structural decoration only: never bake sample readings into artwork.
 art='<defs><pattern id="dots" width="12" height="12" patternUnits="userSpaceOnUse"><circle cx="6" cy="6" r="1" fill="#131313"/></pattern></defs>'
 for x,w in [(28,502),(546,400),(962,320),(1298,320),(1634,258)]:
  art+=f'<rect x="{x}" y="28" width="{w}" height="424" rx="22" fill="#060606" stroke="#1B1B1B"/><rect x="{x}" y="28" width="{w}" height="424" rx="22" fill="url(#dots)"/>'
 for x,w in [(571,350),(987,270),(1323,270)]:art+=f'<path d="M{x} 370h{w}" stroke="#222" stroke-dasharray="2 4"/>'
 # Logo from the supplied vector artwork, positioned as on the board.
 import re
 board=(ROOT/'.jdi/phases/studio-ux-v2/design/boards/ThemePixel.html').read_text(encoding='utf-8')
 logo=re.search(r'<svg viewBox="0 0 19 27"[^>]*>(.*?)</svg>',board,re.S).group(1)
 art+='<svg x="53" y="51" width="15" height="21" viewBox="0 0 19 27">'+logo+'</svg>'
 t.svg('Pixel panels and dot texture',art)
 t.text('Brand',(78,51,200,22),'BEZEL EVO',15,weight=600,spacing=2.4)
 t.clock('Clock',(52,94,390,130),'%H:%M',136,font='Doto',weight=900)
 t.clock('Seconds',(455,106,50,46),'%S',40,orange,'Doto',weight=900)
 t.clock('Weekday',(53,237,145,28),'%A',17,spacing=1)
 t.clock('Date',(205,237,205,28),'%d.%m.%Y',17,grey,spacing=1)
 t.text('CPU heading',(571,51,40,22),'CPU',15,weight=600,spacing=2)
 t.sensor('CPU model',(615,51,280,22),'cpu.name',14,t.muted)
 t.sensor('CPU load',(571,100,160,104),'cpu.usage',78,font='Doto')
 t.bar('CPU block matrix',(816,94,108,108),'cpu.usage',orange,'#202020ff',visualization=dict(style='matrix',columns=4,rows=4,gap=5,radius=4))
 t.graph('CPU history',(571,225,350,67),'cpu.usage','#8A857Cff',style='bars',history=60,visualization=dict(style='rounded',gap=2,radius=2,barWidth=4))
 t.text('GPU heading',(987,51,40,22),'GPU',15,weight=600,spacing=2)
 t.sensor('GPU model',(1032,51,210,22),'gpu.name',14,t.muted)
 t.ring('GPU pixel arc',(1006,116,232,232),'gpu.usage',orange,'#1C1C1Cff',10,start=-90,sweep=180,visualization=dict(style='pixel',ticks=25,radius=0))
 t.sensor('GPU load',(1050,154,148,82),'gpu.usage',72,font='Doto')
 t.text('GPU load caption',(1062,231,120,21),'CARICO',13,t.muted,align='center',spacing=2)
 for x,items in [(571,[('TEMP','cpu.temperature',0),('CLOCK','cpu.frequency',0),('POWER','cpu.power',0)]),(987,[('TEMP','gpu.temperature',0),('VRAM','gpu.memory.used',1),('W','gpu.power',0)])]:
  for i,(label,key,dec) in enumerate(items):
   xx=x+i*(117 if x==571 else 90)
   t.text(label+' caption',(xx,381,100,20),label,13,t.muted,spacing=1.6)
   t.sensor(label+' reading',(xx,402,110 if x==571 else 84,28),key,18,decimals=dec)
 t.text('Memory heading',(1323,51,45,22),'RAM',15,weight=600,spacing=2)
 t.sensor('Memory total',(1371,51,220,22),'memory.total',14,t.muted)
 t.sensor('Memory usage',(1323,86,210,90),'memory.percent',78,font='Doto')
 t.bar('Memory block matrix',(1321,189,274,104),'memory.percent',orange,'#222220ff',visualization=dict(style='matrix',columns=16,rows=6,gap=3,radius=2))
 for y,label,key,c in [(383,'in uso','memory.used',orange),(409,'libera','memory.available.percent',grey)]:
  t.shape('Memory legend',(1323,y+7,9,9),c,2)
  t.text('Memory '+label,(1339,y,115,23),label,15,c if y==409 else None)
  t.sensor('Memory '+key,(1440,y,154,23),key,15,grey if y==409 else None,decimals=1,align='right')
 t.text('Network heading',(1659,51,200,22),'NET',15,weight=600,spacing=2)
 t.sensor('Download',(1678,83,194,64),'net.down',54,font='Doto',decimals=1)
 t.sensor('Upload',(1678,142,194,42),'net.up',32,grey,'Doto',decimals=1)
 t.text('Download direction',(1657,111,18,23),'↓',18,orange)
 t.text('Upload direction',(1657,151,18,23),'↑',18,grey)
 t.graph('Network mirrored history',(1659,203,208,88),'net.down',orange,style='bars',max=20*1024*1024,history=24,visualization=dict(style='mirrored',secondary=dict(key='net.up',min=0,max=20*1024*1024),secondaryColor=grey,gap=3,radius=2,barWidth=6))
 t.text('Volume caption',(53,362,180,24),'VOLUME',13,t.muted,spacing=2)
 t.sensor('Volume',(430,362,74,24),'system.volume',13,align='right')
 t.bar('Output volume blocks',(53,390,452,14),'system.volume','#D8D2C8ff','#202020ff',segments=dict(count=24,gap=5))
 for i,label in enumerate(['0','25','50','75','100']):t.text('Volume scale '+label,(53+i*104,413,36,17),label,11,t.muted)
 t.save('pixel-wide')


def weather(t, box, size=20, color=None, icon=True):
 return t.text('Current weather',box,dict(type='weather',city='Milano',latitude=45.4642,longitude=9.19,showIcon=icon,iconStyle='outline',iconColor=color or t.ink,iconSize=40,iconGap=14,iconMotion='gentle'),size,color)


def player(t,name,box,size=24,**options):
 italic=options.pop('italic',False)
 font=options.pop('font',None)
 weight=options.pop('weight',400)
 color=options.pop('color',None)
 content=dict(type='player',showCover=False,showProgress=False,showTitle=False,showArtist=False,showTime=False,showSource=False)
 content.update(options)
 e=t.text(name,box,content,size,color,font,weight)
 if italic:e['kind']['style']['font']['italic']=True
 return e


def nebula():
 t=Theme('Nebula','#04050Bff','#EEF0FAff','#7C82A6ff','Space Grotesk')
 art='<defs>'
 for n,c,op in [(0,'#7C5CFF',.20),(1,'#22D3EE',.14),(2,'#F472B6',.14)]:
  art+=f'<radialGradient id="glow{n}"><stop stop-color="{c}" stop-opacity="{op}"/><stop offset=".62" stop-color="{c}" stop-opacity="0"/></radialGradient>'
 art+='</defs>'
 for n,(x,y,r) in enumerate([(740,60,320),(1290,470,310),(1800,0,300)]):art+=f'<circle cx="{x}" cy="{y}" r="{r}" fill="url(#glow{n})"/>'
 for x,y,w,h in [(28,170,360,282),(404,28,400,424),(820,28,400,424),(1236,28,400,424),(1652,28,240,424)]:
  art+=f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="26" fill="#FFFFFF" fill-opacity=".035" stroke="#FFFFFF" stroke-opacity=".09"/>'
 art+='<path d="M49 259h318" stroke="#FFFFFF" stroke-opacity=".07"/>'
 t.svg('Nebula glass panels and ambient light',art)
 t.clock('Date',(34,36,342,24),'%A %e %B',14,t.muted,weight=500,spacing=2)
 t.clock('Clock',(34,61,222,94),'%H:%M',96,weight=600,spacing=-4)
 t.clock('Seconds',(257,107,55,42),'%S',30,t.muted,weight=600)
 weather(t,(52,190,312,64),18,'#A5B4FCff')
 for i,(label,key) in enumerate([('CPU','cpu.temperature'),('GPU','gpu.temperature'),('RAM','memory.percent'),('↓','net.down'),('↑','net.up')]):
  x=48+i*64
  t.text(label+' weather footer label',(x,270,59,21),label,13,t.muted,align='center')
  t.sensor(label+' weather footer value',(x,294,59,28),key,17,align='center')
 configs=[(404,'CPU','cpu.usage','cpu.name','#7C5CFFff','#38BDF8ff',[('Temperatura','cpu.temperature',0),('Frequenza','cpu.frequency',0),('Potenza','cpu.power',0)]),
          (820,'GPU','gpu.usage','gpu.name','#F472B6ff','#A78BFAff',[('Temperatura','gpu.temperature',0),('VRAM','gpu.memory.used',1),('Potenza','gpu.power',0)]),
          (1236,'RAM','memory.percent','memory.total','#2DD4BFff','#22D3EEff',[('In uso','memory.used',1),('Disponibile','memory.available.percent',0),('Totale','memory.total',0)])]
 for x,label,key,model,c1,c2,stats in configs:
  t.ring(label+' dial',(x+18,145,176,176),key,dict(scale='arc',start=c2,end=c1,transition=75),'#141930ff',10,start=-135,visualization=dict(style='dial',ticks=41,radius=4,marker=True))
  e=t.sensor(label+' load',(x+42,197,128,62),key,52,weight=600,align='center');e['kind']['style']['parts']=dict(unit=dict(sizePercent=42,color=t.muted))
  t.text(label+' caption',(x+56,257,100,22),label,13,t.muted,align='center',spacing=2)
  t.sensor(label+' model',(x+205,149,174,24),model,16,'#C9CDE6ff',weight=500)
  t.graph(label+' history',(x+205,181,174,46),key,c1,style='area')
  for i,(caption,k,dec) in enumerate(stats):
   y=237+33*i
   t.text(label+' '+caption,(x+205,y,98,22),caption,15,t.muted)
   t.sensor(label+' '+k,(x+281,y,98,24),k,18,font='JetBrains Mono',decimals=dec,align='right')
 t.text('Network caption',(1673,48,190,24),'RETE',14,t.muted,spacing=2)
 t.sensor('Download',(1673,77,196,44),'net.down',34,weight=600,decimals=1)
 t.sensor('Upload',(1673,125,196,28),'net.up',20,t.muted,decimals=1)
 t.graph('Network mirrored history',(1673,285,198,104),'net.down','#22D3EEff',style='area',max=20*1024*1024,history=24,visualization=dict(style='mirrored',secondary=dict(key='net.up',min=0,max=20*1024*1024),secondaryColor='#F472B6ff',seriesGap=0))
 t.save('nebula-wide')


def paper():
 serif='Fraunces 144pt'
 t=Theme('Paper','#EEE8DCff','#1B1915ff','#6F685Cff','Instrument Sans');accent='#B83A12ff'
 art='<path d="M48 72h1824M48 76h1824" stroke="#1B1915" stroke-width="2"/><path d="M548 100v310M1189 100v310" stroke="#C9C0AF"/><path d="M48 426h1824" stroke="#1B1915"/>'
 for y in [175,234,293,352]:art+=f'<path d="M1230 {y}h642" stroke="#D6CDBC"/>'
 t.svg('Paper editorial rules',art)
 t.text('Masthead',(48,23,90,39),'Bezel',30,font=serif,weight=600)
 e=t.text('Edition',(120,23,500,39),'· edizione della sera',30,font=serif,weight=300);e['kind']['style']['font']['italic']=True
 t.clock('Edition date',(1320,36,426,24),'%A %e %B %Y',13,t.muted,align='right',spacing=2)
 t.text('Location',(1768,36,104,24),'MILANO',13,accent,align='right',spacing=2)
 for x,w,title in [(48,460,"L'ORA"),(589,560,'IL TEMPO'),(1230,642,'INDICI DI SISTEMA')]:t.text(title,(x,98,w,24),title,13,accent,weight=600,spacing=3)
 t.clock('Clock',(48,117,460,140),'%H:%M',152,font=serif,weight=300,spacing=-5)
 e=t.clock('Full date',(48,269,460,64),'%A %e %B %Y',22,t.muted,serif,weight=300);e['kind']['style']['font']['italic']=True
 e=t.sensor('Current temperature',(589,125,100,96),'weather.45.464200:9.190000.temperature',76,font=serif,weight=300,unit=False)
 e['kind']['content']['suffix']='°'
 weather(t,(695,145,454,84),22,icon=False)
 t.text('Thermal history caption',(589,239,560,24),'Temperatura CPU · ultimo minuto',16,t.muted)
 t.graph('Thermal history',(589,274,560,61),'cpu.temperature',t.ink,max=100,history=60)
 t.shape('History baseline',(589,335,560,1),'#C9C0AFff')
 t.sensor('CPU thermal reading',(589,340,130,28),'cpu.temperature',18,font=serif)
 t.text('History duration',(1000,340,149,28),'60 s',13,t.muted,align='right')
 for y,label,key,max,dec in [(131,'Processore','cpu.usage',100,0),(190,'Scheda video','gpu.usage',100,0),(249,'Memoria','memory.percent',100,0),(308,'Rete','net.down',20*1024*1024,1)]:
  t.text(label+' caption',(1230,y,120,37),label,18,weight=500)
  t.graph(label+' history',(1366,y+4,120,28),key,t.ink,max=max,history=24)
  t.sensor(label+' reading',(1500,y,280,40),key,30,font=serif,decimals=dec,align='right')
 # Replace unavailable deltas with current related readings, preserving the last column.
 for y,label,key in [(136,'CPU temperature','cpu.temperature'),(195,'GPU temperature','gpu.temperature'),(254,'Available memory','memory.available.percent'),(313,'Upload','net.up')]:t.sensor(label,(1788,y,84,26),key,15,accent,align='right')
 t.text('Media caption',(48,436,136,25),'ORA IN ONDA',13,accent,weight=600,spacing=2)
 player(t,'Current track',(185,434,350,28),18,font=serif,italic=True,showTitle=True,emptyText='In attesa di musica')
 player(t,'Current artist',(550,434,215,28),18,font=serif,italic=True,showArtist=True)
 t.sensor('Disk read',(938,436,150,25),'disk.read',15,unit=True)
 t.text('Disk read caption',(794,436,140,25),'LETTURA DISCO',10,t.muted,spacing=1)
 t.sensor('GPU temperature footer',(1120,436,180,25),'gpu.temperature',15)
 t.sensor('CPU fan',(1330,436,230,25),'cpu.fan',15)
 t.save('paper-wide')


def now_playing():
 t=Theme('Now Playing','#0C070Eff','#FBF4F7ff','#B8A7B2ff','Bricolage Grotesque 96pt');pink='#F472B6ff'
 art='<defs><radialGradient id="halo"><stop stop-color="#DB2777" stop-opacity=".30"/><stop offset=".6" stop-color="#DB2777" stop-opacity="0"/></radialGradient><clipPath id="cover"><rect x="40" y="80" width="320" height="320" rx="20"/></clipPath></defs><circle cx="300" cy="180" r="500" fill="url(#halo)"/>'
 art+='<circle cx="300" cy="240" r="148" fill="#141014"/>'
 for r in range(4,149,4):art+=f'<circle cx="300" cy="240" r="{r}" fill="none" stroke="#1D171D" stroke-width="2"/>'
 art+='<circle cx="300" cy="240" r="50" fill="#DB2777"/><circle cx="300" cy="240" r="7" fill="#0C070E"/><g clip-path="url(#cover)"><rect x="40" y="80" width="320" height="320" fill="#2B0F3A"/><circle cx="110" cy="340" r="140" fill="#DB2777"/><circle cx="300" cy="130" r="110" fill="#F59E0B" fill-opacity=".9"/><circle cx="208" cy="270" r="44" fill="none" stroke="#FBF4F7" stroke-width="12" stroke-opacity=".85"/></g>'
 art+='<rect x="1580" y="91" width="300" height="298" rx="24" fill="#FFFFFF" fill-opacity=".04" stroke="#FFFFFF" stroke-opacity=".08"/><path d="M1605 224h250" stroke="#FFFFFF" stroke-opacity=".08"/>'
 for x,w in [(490,150),(648,110),(766,145)]:art+=f'<rect x="{x}" y="95" width="{w}" height="31" rx="16" fill="#FFFFFF" fill-opacity=".07" stroke="#FFFFFF" stroke-opacity=".09"/>'
 t.svg('Vinyl and standby artwork',art)
 player(t,'Session source',(501,97,128,28),15,showSource=True,emptyText='Musica',weight=500)
 t.text('Audio source label',(660,97,87,28),'AUDIO',13,color='#E9DCE4ff',weight=500)
 player(t,'Remaining time',(778,97,120,28),13,showTime=True,timeFormat='remaining',font='JetBrains Mono')
 player(t,'Track title',(490,141,1050,74),62,showTitle=True,emptyText='In attesa di musica',weight=800)
 player(t,'Artist',(490,216,1050,43),26,showArtist=True,color='#E2D2DCff',weight=500)
 # Real session artwork replaces the local standby geometry whenever a cover exists.
 player(t,'Album cover',(40,80,320,320),24,showCover=True,coverRadius=20,emptyText='')
 t.text('System audio spectrum',(490,278,1050,74),dict(type='spectrum',bands=64,low=20,high=16000,audio='system',style='mirrored',centre=True,demo=False,thickness=7,gap=3,corners=[2,2,2,2]),20,pink)
 player(t,'Playback timeline',(490,359,1050,28),15,showTime=True,showProgress=True,font='JetBrains Mono',color='#A8959Fff',progressStyle=dict(fill=pink,track='#FFFFFF18',thickness=3,times='sides'))
 t.clock('Clock',(1605,107,250,75),'%H:%M',76,weight=700,spacing=-3)
 t.clock('Date',(1605,186,250,30),'%A %e %B',17,t.muted)
 for y,label,key,bar_key in [(239,'CPU','cpu.usage','cpu.usage'),(291,'GPU','gpu.temperature','gpu.temperature'),(343,'RAM','memory.used','memory.percent')]:
  t.text(label+' caption',(1605,y,52,23),label,15,t.muted)
  t.sensor(label+' reading',(1690,y,165,23),key,15,font='JetBrains Mono',decimals=1 if label=='RAM' else 0,align='right')
  t.bar(label+' meter',(1605,y+29,250,4),bar_key,pink,'#FFFFFF14')
 t.save('now-playing-wide')


if __name__=='__main__':
 for create in [pixel,nebula,paper,now_playing]:create()
