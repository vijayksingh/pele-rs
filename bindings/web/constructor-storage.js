// Private bounded constructor wire interpreter. Storage and admission policy are
// bound once; this module knows no diagram, prototype registry or public owner.
export const invalidConstructorBatch=()=>{throw new TypeError('Invalid host constructor batch');};
export function createConstructorDecoder(sink){
  const {rawAtoms,stringRoom,addString,nodeCount,beginNode,mapEntry:writeMap,
    setValue:writeSet,propertyKey,property:writeProperty,endNode:finishNode,
    layoutKey,saveLayout,record,dense,
    reference,number}=sink;
  return function decode(state,bytes){
    const bad=invalidConstructorBatch;
    if(!(bytes instanceof Uint8Array)||bytes.byteLength<2||bytes.byteLength>65536)bad();
    const view=new DataView(bytes.buffer,bytes.byteOffset,bytes.byteLength);
    const count=view.getUint16(0,true);if(count<1||count>128)bad();
    let offset=2,atomTag=0,atomLow=0,atomHigh=0;
    const need=n=>{if(n>bytes.byteLength-offset)bad();};
    const u8=()=>{need(1);return view.getUint8(offset++);};
    const u32=()=>{need(4);const value=view.getUint32(offset,true);offset+=4;return value;};
    const scratch=[];
    const text=units=>{
      need(units*2);const parts=[];
      for(let left=units;left>0;){
        const size=Math.min(left,8192);scratch.length=size;
        for(let i=0;i<size;i++){scratch[i]=view.getUint16(offset,true);offset+=2;}
        parts.push(String.fromCharCode(...scratch));left-=size;
      }
      return parts.join('');
    };
    // Scalar registers, never an Atom wrapper or per-atom argument array. Live
    // resolution happens here, before any subsequent atom/field is consumed.
    const atom=()=>{
      atomTag=u8();atomLow=0;atomHigh=0;
      switch(atomTag){
        case 0:return undefined;case 1:return null;case 2:return false;case 3:return true;
        case 4:{
          need(8);
          if(rawAtoms){atomLow=view.getUint32(offset,true);atomHigh=view.getUint32(offset+4,true);offset+=8;return undefined;}
          const value=number(view,offset);offset+=8;return value;
        }
        case 5:case 6:case 7:case 8:atomLow=u32();return rawAtoms?undefined:reference(state,atomTag,atomLow);
        default:bad();
      }
    };
    // Bulk storage primitives consume the shared atom reader. Live sinks receive
    // real values directly; column sinks reuse one scalar register per frame.
    // Neither path allocates a temporary record or calls a sink per field.
    const registers=rawAtoms?{tag:0,lo:0,hi:0}:undefined;
    const specialAtom=rawAtoms?()=>{
      atom();registers.tag=atomTag;registers.lo=atomLow;registers.hi=atomHigh;
    }:atom;
    const startNode=(id,kind,length)=>{
      if(state.constructionNode||state.constructionString||kind>4||id!==nodeCount(state))bad();
      state.constructionNode=beginNode(state,id,kind,length);
    };
    const mapEntry=()=>{
      const node=state.constructionNode;if(!node||node.kind!==2||node.properties||state.constructionString)bad();
      const key=atom(),keyTag=atomTag,keyLow=atomLow,keyHigh=atomHigh;
      const value=atom();writeMap(state,node,keyTag,keyLow,keyHigh,key,atomTag,atomLow,atomHigh,value);
    };
    const setValue=()=>{
      const node=state.constructionNode;if(!node||node.kind!==3||node.properties||state.constructionString)bad();
      const value=atom();writeSet(state,node,atomTag,atomLow,atomHigh,value);
    };
    const property=()=>{
      const node=state.constructionNode;if(!node||state.constructionString)bad();
      const key=u32(),flags=u8(),name=propertyKey(state,key),value=atom();node.properties=true;
      return writeProperty(state,node,key,name,flags,atomTag,atomLow,atomHigh,value);
    };
    const endNode=()=>{
      const node=state.constructionNode;if(!node||state.constructionString)bad();
      if(!finishNode(state,node))return false;
      state.constructionNode=null;return true;
    };
    for(let event=0;event<count;event++){
      const tag=u8();
      switch(tag){
        case 0:{
          if(state.constructionNode||state.constructionString)bad();
          const id=u32(),units=u32();stringRoom(state,id,units);
          addString(state,id,text(units));break;
        }
        case 1:{
          if(state.constructionNode||state.constructionString)bad();
          const id=u32(),units=u32();stringRoom(state,id,units);
          state.constructionString={id,units,read:0,parts:[]};break;
        }
        case 2:{
          const pending=state.constructionString;if(!pending||state.constructionNode)bad();
          const units=u32();if(units===0||units>pending.units-pending.read)bad();
          pending.parts.push(text(units));pending.read+=units;break;
        }
        case 3:{
          const pending=state.constructionString;
          if(!pending||pending.read!==pending.units||state.constructionNode)bad();
          addString(state,pending.id,pending.parts.join(''));state.constructionString=null;break;
        }
        case 4:{
          if(state.constructionNode||state.constructionString)bad();
          const id=u32(),kind=u8(),length=u32();startNode(id,kind,length);break;
        }
        case 5:mapEntry();break;
        case 6:setValue();break;
        case 7:if(!property())return -1;break;
        case 8:if(!endNode())return -2;break;
        case 9:{
          if(state.constructionNode||state.constructionString)bad();
          const id=u32(),kind=u8(),length=u32(),payloadCount=u32(),propertyCount=u32();
          if(kind>4||(kind!==2&&kind!==3&&payloadCount!==0))bad();
          need(payloadCount*(kind===2?2:1)+propertyCount*6);
          startNode(id,kind,length);
          if(kind===2)for(let i=0;i<payloadCount;i++)mapEntry();
          else if(kind===3)for(let i=0;i<payloadCount;i++)setValue();
          for(let i=0;i<propertyCount;i++)if(!property())return -1;
          if(!endNode())return -2;break;
        }
        case 10:{
          if(state.constructionNode||state.constructionString)bad();
          const id=u8(),count=u8();
          if(id!==state.constructionLayouts.length||id>=32||count>64)bad();
          need(count*4);
          const keys=new Array(count),ids=rawAtoms?new Array(count):null;
          for(let i=0;i<count;i++){
            const keyId=u32(),key=layoutKey(state,keyId);
            if(typeof key!=='string'||key==='__proto__'||key==='length'||keys.includes(key))bad();
            keys[i]=key;if(ids)ids[i]=keyId;
          }
          state.constructionLayouts.push(saveLayout(state,keys,ids));break;
        }
        case 11:{
          if(state.constructionNode||state.constructionString)bad();
          const id=u32(),layout=u8(),keys=state.constructionLayouts[layout];
          if(!keys||id!==nodeCount(state))bad();
          const length=keys.length;need(length);
          const status=record(state,id,keys,specialAtom,registers);
          if(status!==0)return status;break;
        }
        case 12:{
          if(state.constructionNode||state.constructionString)bad();
          const id=u32(),length=u32();
          if(id!==nodeCount(state))bad();
          need(length);
          const status=dense(state,id,length,specialAtom,registers);
          if(status!==0)return status;break;
        }
        default:bad();
      }
    }
    if(offset!==bytes.byteLength)bad();
    return 0;
  };
}
