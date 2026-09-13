"""Bounded read-only ring snapshots to investigate concurrent enemy animation tracks."""
import argparse
import importlib.util
import json
from pathlib import Path
import struct
import time

ROOT=Path(__file__).resolve().parent.parent
spec=importlib.util.spec_from_file_location('live',ROOT/'scripts/inspect-live-game.py')
live=importlib.util.module_from_spec(spec);spec.loader.exec_module(live)
parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('pid',type=int)
parser.add_argument('--seconds',type=float,default=20)
parser.add_argument('--handle',type=lambda value:int(value,0),help='Read one previously observed actor even when unlocked')
args=parser.parse_args()
if not 0<args.seconds<=30:raise ValueError('Capture limited to 30 seconds')
process=live.Process(args.pid)
output=ROOT/'dist/review-0.6'/f'animation-tracks-{args.pid}-{time.time_ns()}.jsonl'
try:
    (ROOT/'dist/review-0.6/animation-writer.bin').write_bytes(process.read(process.base+0xb5c580,1536))
    end=time.perf_counter()+args.seconds
    with output.open('x') as stream:
        while time.perf_counter()<end:
            row=dict(unix_ns=time.time_ns())
            try:
                world=process.pointer(process.base+0x3d7a1e0)
                if args.handle is None:
                    target=process.locked_target(world)
                else:
                    group,index=(args.handle>>14)&63,args.handle&16383
                    bucket=process.pointer(world+0x518+group*8)
                    count=struct.unpack('<i',process.read(bucket,4))[0]
                    if not 0<=index<count<=16384:raise ValueError('Handle index invalid')
                    actor=process.pointer(process.pointer(bucket+8)+index*0x38)
                    sample=process.actor(actor)
                    if int(sample['handle'],16)!=args.handle:raise ValueError('Actor replaced')
                    target=dict(actor=sample)
                if target:
                    actor=int(target['actor']['address'],16)
                    modules=process.pointer(actor+0x1ff8)
                    module=process.pointer(modules+0x10)
                    raw=process.read(module+0x20,224)
                    head=struct.unpack_from('<i',raw,200)[0]
                    frames=[struct.unpack_from('<ifffi',raw,20*((head+9-i)%10)) for i in range(10)]
                    if process.read(module+0x20,224)!=raw:raise ValueError('Ring changed during read')
                    if process.pointer(actor+0x1ff8)!=modules:raise ValueError('Owner changed during read')
                    row.update(model=target['actor']['character_id']//10000,handle=target['actor']['handle'],head=head,frames=frames,controls=struct.unpack_from('<6i',raw,200))
            except (OSError,ValueError) as error:row['error']=str(error)
            stream.write(json.dumps(row)+'\n');time.sleep(.010)
    print(output)
finally:process.close()
