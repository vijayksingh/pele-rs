// Native canonical epochs are the only graph store. This adapter reads pinned
// POD views and emits bounded observations; graph algorithms stay in Rust.
const PAGE=1024,DIRECTORY=256,ABSENT=0xffffffff,PROPERTY=1,ITEM=1+2**26,ARGUMENT=1+2*2**26;
const kinds=['object','instance','array','map','set','date','error'];
const errors=['error','pele','type','range','syntax','reference','eval','uri'];
const invalid=()=>{throw new TypeError('Invalid native canonical storage');};
const u32=value=>Number.isInteger(value)&&value>=0&&value<=0xffffffff;
function missingKind(){const node=undefined;return node.kind;}
const bits=new DataView(new ArrayBuffer(8));
const hex=(lo,hi)=>hi.toString(16).padStart(8,'0')+lo.toString(16).padStart(8,'0');
function decodeNumber(lo,hi){bits.setUint32(0,lo,true);bits.setUint32(4,hi,true);return bits.getFloat64(0,true);}

export class NativeGraphView {
  #epoch;#memory;#descriptor;#buffer;#view;#refs=1;#strings=null;#prefix=0;#owner;
  // Four row IDs followed by copied Header7/Property5/Item3/StringDesc2 words.
  // These are immutable scalar values, never retained views of native pages.
  #rows=new Uint32Array(21);#units=null;
  constructor(epoch,memory,owner){
    this.#epoch=epoch;this.#memory=memory;this.#owner=owner;this.#rows.fill(ABSENT,0,4);
    try {
    const d=owner.callNative(()=>epoch.descriptor());
    if(!(d instanceof Uint32Array)||d.length!==17||d[0]!==1||d[1]!==PAGE)invalid();
    this.#descriptor=Array.from(d);this.#refresh();
    for(let i=9;i<=13;i++)this.#range(d[i],DIRECTORY*4,4);
    this.#range(d[14],d[16]*16,4);
    if(d[16]>d[15]||d[4]>8)invalid();
    }catch(error){this.#epoch=null;try{epoch.close();}finally{epoch.free?.();}throw error;}
  }
  get epoch(){if(!this.#epoch)invalid();return this.#epoch;}
  get owner(){if(!this.#epoch)invalid();return this.#owner;}
  get nodeCount(){if(!this.#epoch)invalid();return this.#descriptor[7];}
  get stringCount(){if(!this.#epoch)invalid();return this.#descriptor[8];}
  retain(){if(!this.#epoch)invalid();this.#refs++;return this;}
  release(){if(--this.#refs<0)invalid();if(this.#refs===0){const epoch=this.#epoch;this.#epoch=null;this.#view=null;this.#buffer=null;this.#strings=null;this.#descriptor=null;this.#memory=null;this.#owner=null;this.#rows=null;this.#units=null;try{epoch.close();}finally{epoch.free?.();}}}
  #refresh(){if(!this.#epoch)invalid();const buffer=this.#memory.buffer;if(buffer!==this.#buffer){this.#buffer=buffer;this.#view=new DataView(buffer);}return this.#view;}
  #range(pointer,bytes,alignment=1){if(!u32(pointer)||!Number.isSafeInteger(bytes)||bytes<0||pointer%alignment||pointer+bytes>this.#buffer.byteLength)invalid();}
  // Caller has refreshed once and cannot invoke user code before finishing the
  // copy. Return only an address; no page/record view escapes this read window.
  #address(view,table,row,width,count=1,unitBytes=4){
    const page=Math.floor(row/PAGE),top=this.#descriptor[table];
    const directory=view.getUint32(top+Math.floor(page/DIRECTORY)*4,true);
    if(!directory)invalid();this.#range(directory,DIRECTORY*8,4);
    const entry=directory+(page%DIRECTORY)*8,pointer=view.getUint32(entry,true),length=view.getUint32(entry+4,true),offset=row%PAGE;
    if(!pointer||length>PAGE*width||(offset+count)*width>length)invalid();this.#range(pointer,length*unitBytes,unitBytes);
    return pointer+offset*width*unitBytes;
  }
  #word(table,row,width,column){
    if(!this.#epoch||!u32(row)||row>=2**26||!Number.isInteger(column)||column<0||column>=width)invalid();
    const cursor=table-9,start=table===9?4:table===10?11:table===11?16:19;
    if(cursor<0||cursor>3||width!==(table===9?7:table===10?5:table===11?3:2))invalid();
    if(this.#rows[cursor]!==row){
      const view=this.#refresh(),address=this.#address(view,table,row,width);
      this.#rows[cursor]=ABSENT;
      for(let i=0;i<width;i++)this.#rows[start+i]=view.getUint32(address+i*4,true);
      this.#rows[cursor]=row;
    }
    return this.#rows[start+column];
  }
  #copyUnits(row,count){
    if(!this.#epoch||!u32(row)||row>=2**26||!u32(count)||count===0||count>PAGE-row%PAGE||row+count>2**26)invalid();
    const view=this.#refresh(),address=this.#address(view,13,row,1,count,2);
    const units=this.#units??(this.#units=[]);units.length=count;
    for(let i=0;i<count;i++)units[i]=view.getUint16(address+i*2,true);
    return units;
  }
  headerWord(id,column){if(!u32(id)||id>=this.nodeCount)invalid();return this.#word(9,id,7,column);}
  kindCode(id){return this.headerWord(id,0);}
  propertyAddress(id,slot){const count=this.headerWord(id,4);if(!u32(slot)||slot>=count)invalid();return this.headerWord(id,3)+slot;}
  propertyWord(address,column){return this.#word(10,address,5,column);}
  itemAddress(id,slot){const count=this.headerWord(id,6)*(this.kindCode(id)===3?2:1);if(!u32(slot)||slot>=count)invalid();return this.headerWord(id,5)+slot;}
  atomWord(slot,column){
    if(!this.#epoch||!Number.isInteger(column)||column<0||column>2)invalid();
    if(slot===0)return this.#descriptor[4+column];
    if(!Number.isSafeInteger(slot)||slot<1)invalid();
    if(slot<ITEM)return this.#word(10,slot-PROPERTY,5,column+2);
    if(slot<ARGUMENT)return this.#word(11,slot-ITEM,3,column);
    const row=slot-ARGUMENT;if(row>=this.#descriptor[16])invalid();
    return this.#refresh().getUint32(this.#descriptor[14]+row*16+4+column*4,true);
  }
  argumentValues(){
    if(!this.#epoch)invalid();
    const out=new Array(this.#descriptor[15]);let previous=-1;
    for(let row=0;row<this.#descriptor[16];row++){
      const index=this.#refresh().getUint32(this.#descriptor[14]+row*16,true);
      if(index<=previous||index>=out.length)invalid();previous=index;out[index]=ARGUMENT+row;
    }
    return out;
  }
  releaseArguments(){this.#owner.callNative(()=>this.epoch.release_arguments());this.#descriptor[14]=0;this.#descriptor[15]=0;this.#descriptor[16]=0;}
  useStrings(strings,prefix){if(!Array.isArray(strings)||!u32(prefix)||prefix>strings.length||prefix>this.stringCount)invalid();this.#strings=strings;this.#prefix=prefix;}
  stringValueAt(id){
    if(!u32(id)||id>=this.stringCount)invalid();
    if(id<this.#prefix)return this.#strings[id];
    const start=this.#word(12,id,2,0),length=this.#word(12,id,2,1),parts=[];
    // Only bounded transport scratch, never a second UTF16 model table.
    for(let at=0;at<length;){
      const count=Math.min(PAGE-(start+at)%PAGE,length-at);
      // Spread copies all arguments before fromCharCode can reenter. A nested
      // operation may grow memory or overwrite this scratch without invalidating
      // those arguments; the next native chunk always refreshes its own window.
      parts.push(String.fromCharCode(...this.#copyUnits(start+at,count)));at+=count;
    }
    return parts.join('');
  }
  exactEquals(other){return this.#owner.callNative(()=>this.epoch.exact_equals(other.epoch));}
  preflight(bytes){return this.#owner.callNative(()=>this.epoch.preflight_encoding(bytes));}
}

class NativeCapture {
  #owner;#observation;#bytes=new Uint8Array(65536);#view;#offset=8;#events=0;#closed=false;#node=-1;#items=0;#itemKind;
  #number=new DataView(new ArrayBuffer(8));#lo=0;#hi=0;
  constructor(baseline,delta){
    if(delta!==false)invalid();this.#owner=baseline.owner;
    this.#observation=this.#owner.callNative(()=>this.#owner.transport.begin_observation(baseline.epoch));
    this.#view=new DataView(this.#bytes.buffer);
  }
  #flush(){
    if(!this.#events)return;
    this.#view.setUint16(0,1,true);this.#view.setUint16(2,this.#events,true);this.#view.setUint32(4,this.#offset,true);
    this.#owner.callNative(()=>this.#observation.append(this.#bytes.subarray(0,this.#offset)));this.#offset=8;this.#events=0;
  }
  #record(opcode,size){
    if(this.#closed||!u32(size)||size+13>this.#bytes.length)invalid();
    if(this.#events===128||this.#offset+5+size>this.#bytes.length)this.#flush();
    const offset=this.#offset;this.#view.setUint8(offset,opcode);this.#view.setUint32(offset+1,size,true);this.#offset+=5+size;this.#events++;return offset+5;
  }
  #atom(at,tag,payload){
    if(tag===4){this.#number.setFloat64(0,payload,true);this.#lo=this.#number.getUint32(0,true);this.#hi=this.#number.getUint32(4,true);}
    else{this.#lo=tag>=5?payload:0;this.#hi=0;}
    if(!u32(tag)||tag>8||!u32(this.#lo)||!u32(this.#hi))invalid();
    this.#view.setUint8(at,tag);this.#view.setUint32(at+1,this.#lo,true);this.#view.setUint32(at+5,this.#hi,true);
  }
  root(tag,payload){this.#atom(this.#record(9,9),tag,payload);}
  beginArguments(length){if(!u32(length))invalid();this.#view.setUint32(this.#record(10,4),length,true);}
  argument(index,tag,payload){const at=this.#record(11,13);this.#view.setUint32(at,index,true);this.#atom(at+4,tag,payload);}
  beginNode(id){if(this.#node!==-1||!u32(id))invalid();this.#node=id;this.#items=0;this.#itemKind=undefined;this.#view.setUint32(this.#record(4,4),id,true);}
  property(key,flags,tag,payload){const at=this.#record(5,14);this.#view.setUint32(at,key,true);this.#view.setUint8(at+4,flags);this.#atom(at+5,tag,payload);}
  beginItems(kind){if(kind!=='map'&&kind!=='set')invalid();this.#itemKind=kind;}
  mapEntry(keyTag,key,valueTag,value){const at=this.#record(6,18);this.#atom(at,keyTag,key);this.#atom(at+9,valueTag,value);this.#items++;}
  setItem(tag,payload){this.#atom(this.#record(7,9),tag,payload);this.#items++;}
  finishNode(id,kind,aux){
    if(id!==this.#node)invalid();const code=kinds.indexOf(kind);let lo=0,hi=0;if(code<0)invalid();
    if(code===1){if(!u32(aux))invalid();lo=aux;}
    else if(code===2){if(u32(aux))lo=aux;else hi=1;}
    else if(code===3||code===4){if(this.#itemKind!==kind)invalid();lo=this.#items;}
    else if(code===5){if(typeof aux!=='string'||!/^[0-9a-fA-F]{16}$/.test(aux))invalid();lo=parseInt(aux.slice(8),16);hi=parseInt(aux.slice(0,8),16);}
    else if(code===6){const brand=errors.indexOf(aux);if(brand<0)hi=1;else lo=brand;}
    const at=this.#record(8,12);this.#view.setUint32(at,code,true);this.#view.setUint32(at+4,lo,true);this.#view.setUint32(at+8,hi,true);this.#node=-1;
  }
  #strings(strings){
    const start=this.#owner.callNative(()=>this.#observation.history_count());if(!u32(start)||start>strings.length)invalid();
    for(let id=start;id<strings.length;id++){
      const value=strings[id],at=this.#record(1,8);this.#view.setUint32(at,id,true);this.#view.setUint32(at+4,value.length,true);
      for(let unit=0;unit<value.length;unit+=1024){const count=Math.min(1024,value.length-unit),to=this.#record(2,count*2);for(let i=0;i<count;i++)this.#view.setUint16(to+i*2,value.charCodeAt(unit+i),true);}
      this.#record(3,0);
    }
  }
  finish(strings){
    if(this.#closed||this.#node!==-1)invalid();this.#strings(strings);this.#flush();
    const epoch=this.#owner.callNative(()=>this.#observation.finish());this.#closed=true;this.#observation.free?.();this.#observation=null;
    const graph=new NativeGraphView(epoch,this.#owner.memory,this.#owner);
    try{graph.useStrings(strings,strings.length);return {graph,values:graph.argumentValues()};}
    catch(error){graph.release();throw error;}
  }
  abort(strings){
    if(this.#closed)return;
    try{this.#strings(strings);this.#flush();}
    finally{this.#closed=true;try{this.#owner.callNative(()=>this.#observation.abort());}finally{this.#observation.free?.();this.#observation=null;}}
  }
}

export const NativeStorage=Object.freeze({
  stringCount:graph=>graph.stringCount,stringValueAt:(graph,id)=>graph.stringValueAt(id),nodeCount:graph=>graph.nodeCount,
  *nodeIds(graph){for(let id=0;id<graph.nodeCount;id++)yield id;},rootSlot:()=>0,
  nodeKind(graph,id){const code=graph.kindCode(id);return code===ABSENT?missingKind():kinds[code];},
  nodeAux(graph,id){const kind=graph.kindCode(id);if(kind===ABSENT)return missingKind();const lo=graph.headerWord(id,1),hi=graph.headerWord(id,2);return kind===5?hex(lo,hi):kind===6?(hi?undefined:errors[lo]):kind===2&&hi?undefined:lo;},
  propertyCount:(graph,id)=>graph.headerWord(id,4),propertySlot:(graph,id,at)=>graph.propertyAddress(id,at),
  *propertySlots(graph,id){const start=graph.headerWord(id,3),count=graph.headerWord(id,4);for(let at=0;at<count;at++)yield start+at;},
  propertyKey:(graph,slot)=>graph.propertyWord(slot,0),propertyFlags:(graph,slot)=>graph.propertyWord(slot,1),propertyAtomSlot:(_graph,slot)=>PROPERTY+slot,
  itemCount:(graph,id)=>graph.headerWord(id,6),itemAtomSlot:(graph,id,at)=>ITEM+graph.itemAddress(id,at),
  atomTag:(graph,slot)=>graph.atomWord(slot,0),atomPayload(graph,slot){const tag=graph.atomWord(slot,0),lo=graph.atomWord(slot,1);return tag===4?decodeNumber(lo,graph.atomWord(slot,2)):lo;},atomNumberBits:(graph,slot)=>hex(graph.atomWord(slot,1),graph.atomWord(slot,2)),
  beginCapture:(baseline,delta=false)=>new NativeCapture(baseline,delta),releaseArguments:graph=>graph.releaseArguments(),
});
