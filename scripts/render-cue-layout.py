"""Rasterize the offline ImGui layout mesh; this is not gameplay evidence."""
import json
import math
from pathlib import Path
import sys

ROOT=Path(__file__).resolve().parent.parent
sys.path.insert(0,str(ROOT/'dist/video-tools'))
from PIL import Image, ImageDraw

folder=Path(sys.argv[1]) if len(sys.argv)>1 else ROOT/'dist/review-0.6/layout'
data=json.loads((folder/'mesh.json').read_text())
aw,ah=data['atlas']
texture=Image.frombytes('RGBA',(aw,ah),(folder/'atlas.rgba').read_bytes()).load()
width,height=map(int,data.get('display',[1920,1080]))
canvas=Image.new('RGB',(width,height),(40,43,46))
pixels=canvas.load()
for mesh in data['lists']:
    vertices,indices=mesh['vertices'],mesh['indices']
    for count,offset,vertex_offset,clip in mesh['commands']:
        for i in range(offset,offset+count,3):
            a,b,c=[vertices[indices[j]+vertex_offset] for j in range(i,i+3)]
            denominator=(b[1]-c[1])*(a[0]-c[0])+(c[0]-b[0])*(a[1]-c[1])
            if abs(denominator)<1e-8: continue
            x0=max(0,math.floor(min(a[0],b[0],c[0])),math.floor(clip[0]))
            x1=min(width,math.ceil(max(a[0],b[0],c[0])),math.ceil(clip[2]))
            y0=max(0,math.floor(min(a[1],b[1],c[1])),math.floor(clip[1]))
            y1=min(height,math.ceil(max(a[1],b[1],c[1])),math.ceil(clip[3]))
            for y in range(y0,y1):
                for x in range(x0,x1):
                    u=((b[1]-c[1])*(x+.5-c[0])+(c[0]-b[0])*(y+.5-c[1]))/denominator
                    v=((c[1]-a[1])*(x+.5-c[0])+(a[0]-c[0])*(y+.5-c[1]))/denominator
                    w=1-u-v
                    if min(u,v,w)<-1e-6: continue
                    tx=min(aw-1,max(0,int((u*a[2]+v*b[2]+w*c[2])*aw)))
                    ty=min(ah-1,max(0,int((u*a[3]+v*b[3]+w*c[3])*ah)))
                    tex=texture[tx,ty]
                    alpha=(u*a[7]+v*b[7]+w*c[7])/255*tex[3]/255
                    old=pixels[x,y]
                    pixels[x,y]=tuple(round(old[k]*(1-alpha)+(u*a[k+4]+v*b[k+4]+w*c[k+4])*tex[k]/255*alpha) for k in range(3))
draw=ImageDraw.Draw(canvas)
caption='0.6.3 DESIGN / OFFLINE RENDER / not gameplay' if width==960 else 'OFFLINE RENDER CHECK / 1080p scale / not gameplay'
draw.text(((width-draw.textlength(caption))/2,20 if width==960 else 45),caption,fill=(235,235,235))
canvas.save(folder/'cue-layout.png')
print(folder/'cue-layout.png')
