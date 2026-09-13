"""Read-only extraction of local Sekiro attack/behavior parameters for response classification."""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import struct
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parent.parent
REF = ROOT / 'dist/game-analysis/references'
spec=importlib.util.spec_from_file_location('archives',Path(__file__).with_name('inspect-game-archives.py'))
archives=importlib.util.module_from_spec(spec); spec.loader.exec_module(archives)


def layout(definition):
    root=ET.parse(definition).getroot()
    fields=[]; offset=0; bit=0; bit_bytes=0
    formats={'u8':'B','s8':'b','u16':'H','s16':'h','u32':'I','s32':'i','f32':'f','dummy8':'B','fixstr':'B','fixstrW':'H'}
    for field in root.find('Fields'):
        match=re.fullmatch(r'(\w+)\s+(\w+)(?::(\d+))?(?:\[(\d+)\])?(?:\s*=.*)?',field.attrib['Def'])
        if not match: raise ValueError(field.attrib['Def'])
        kind,name,bits,count=match.groups(); fmt=formats[kind]; size=struct.calcsize(fmt)
        if bits:
            bits=int(bits)
            if bit and (bit_bytes!=size or bit+bits>size*8): offset+=bit_bytes; bit=0
            bit_bytes=size
            fields.append((name,offset,fmt,bit,bits,1))
            bit+=bits
            if bit==size*8: offset+=size; bit=0
        else:
            if bit: offset+=bit_bytes;bit=0
            count=int(count or 1)
            fields.append((name,offset,fmt,0,0,count))
            offset+=size*count
    if bit: offset+=bit_bytes
    return root.findtext('ParamType'),int(root.findtext('DataVersion')),offset,fields


def parse_param(data, definition):
    expected,version,size,fields=layout(definition)
    if data[0x2c]!=0: raise ValueError('Expected little-endian parameter file')
    flags=data[0x2d]
    data_version,count=struct.unpack_from('<hH',data,8)
    type_offset=struct.unpack_from('<q',data,16)[0] if flags&0x80 else 12
    param_type=data[type_offset:data.index(b'\0',type_offset)].decode('ascii').strip()
    if param_type!=expected or data_version!=version: raise ValueError((param_type,data_version,expected,version))
    table=0x40 if flags&4 or flags&3==3 else 0x30
    stride=24 if flags&4 else 12
    rows=[]
    for i in range(count):
        at=table+i*stride
        row_id=struct.unpack_from('<i',data,at)[0]
        offset=struct.unpack_from('<q' if flags&4 else '<I',data,at+(8 if flags&4 else 4))[0]
        if not table+count*stride<=offset<=offset+size<=len(data): raise ValueError('Parameter row outside bounds')
        name_offset=struct.unpack_from('<q' if flags&4 else '<I',data,at+(16 if flags&4 else 8))[0]
        name=''
        if name_offset:
            if data[0x2e]&1:
                end=name_offset
                while end+2<=len(data) and data[end:end+2]!=b'\0\0': end+=2
                name=data[name_offset:end].decode('utf-16-le')
            else:
                name=data[name_offset:data.index(b'\0',name_offset)].decode('shift_jis')
        rows.append((row_id,offset,name))
    offsets=sorted(set(o for _,o,_ in rows))
    if any(b-a!=size for a,b in zip(offsets,offsets[1:])): raise ValueError(f'Row stride differs from schema {size}')
    result={}
    for row_id,offset,name in rows:
        if row_id in result: raise ValueError('Duplicate parameter row id')
        row={'_name':name}
        for name,pos,fmt,shift,bits,count in fields:
            if count!=1: continue
            value=struct.unpack_from('<'+fmt,data,offset+pos)[0]
            if bits: value=(value>>shift)&((1<<bits)-1)
            row[name]=value
        result[row_id]=row
    return result


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('game_directory',type=Path)
    args=parser.parse_args()
    if hashlib.sha256((args.game_directory/'sekiro.exe').read_bytes()).hexdigest()!=archives.RESEARCH_HASH:
        raise ValueError('Unsupported executable')
    inventory=json.loads((ROOT/'dist/game-analysis-v2/archive-inventory.json').read_text())
    entry=next(e for e in inventory['entries'] if e['paths']==['/param/gameparam/gameparam.parambnd.dcx'])
    contents,source_hash=archives.extract_container(entry,args.game_directory)
    output=ROOT/'dist/game-analysis-v2'
    wanted={'AtkParam_Npc':'AtkParam','BehaviorParam':'BehaviorParam','NpcParam':'NpcParam','ThrowParam':'ThrowParam'}
    sources={}
    for name,data in archives.binder_timelines(contents,args.game_directory,suffix='.param'):
        short=name.replace('\\','/').split('/')[-1].removesuffix('.param')
        if short not in wanted: continue
        definition=REF/f'SDT.{wanted[short]}.xml'
        result=parse_param(data,definition)
        (output/f'{short}.json').write_text(json.dumps(result,indent=2))
        sources[short]=dict(sha256=hashlib.sha256(data).hexdigest(),schema_sha256=hashlib.sha256(definition.read_bytes()).hexdigest(),rows=len(result))
        print(short,len(result))
    if set(sources)!=set(wanted): raise ValueError('Required parameter files missing')
    (output/'attack-param-sources.json').write_text(json.dumps(dict(container_sha256=source_hash,sources=sources),indent=2))


if __name__=='__main__': main()
