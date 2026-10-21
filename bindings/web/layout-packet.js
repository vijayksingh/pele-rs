// Source recipes carry complete accepted state; this is never an unchanged ack.
import {decodeSourceLayout} from './layout-storage.js';
const BUDGET=64*1024*1024,encoder=new TextEncoder();
const invalid=()=>{throw new TypeError('Invalid source layout outcome');};
export function decodeLayoutOutcome(response,baseline,expectedRevision,raw,type){
  if(!Array.isArray(response)||(response.length!==4&&response.length!==6)||response[0]!==1||!Number.isSafeInteger(response[1])||response[1]<33||response[1]>BUDGET||!(response[2] instanceof Uint8Array)||response[2].length>BUDGET)invalid();
  const bytes=response[2],canonicalBytes=response[1];
  if(bytes[1]===69&&bytes.length!==canonicalBytes)invalid();
  let metadataBytes;
  if(response.length===4){if(typeof response[3]!=='string')invalid();metadataBytes=encoder.encode(response[3]).length;}
  else{metadataBytes=response[3];if(!Number.isSafeInteger(metadataBytes)||metadataBytes<0||(response[4]!==0&&response[4]!==1))invalid();}
  if(canonicalBytes+metadataBytes>BUDGET)invalid();
  const exchange=decodeSourceLayout(bytes,baseline);
  if(expectedRevision!==undefined&&BigInt(exchange.revision)!==BigInt(expectedRevision)+1n)invalid();
  const outcome={revision:exchange.revision,graph:exchange.graph};
  if(response.length===4){outcome.result=JSON.parse(response[3]);return outcome;}
  if(response[4]===1){outcome.factoryError=response[5];return outcome;}
  const packet=response[5];if(!Array.isArray(packet)||packet.length!==3||packet[0]!==1||packet[1]!==metadataBytes)invalid();
  const value=packet[2],keys=raw?['svg','width','height','links']:['type','svg','width','height','links'],actual=value&&Object.keys(value);
  if(!value||Object.getPrototypeOf(value)!==Object.prototype||actual.length!==keys.length||!keys.every((key,i)=>actual[i]===key)||typeof value.svg!=='string'||typeof value.width!=='number'||typeof value.height!=='number'||!Array.isArray(value.links)||(!raw&&value.type!==type))invalid();
  for(const link of value.links){const fields=link&&Object.keys(link);if(!link||Object.getPrototypeOf(link)!==Object.prototype||fields.length!==3||fields[0]!=='id'||fields[1]!=='href'||fields[2]!=='internal'||typeof link.id!=='string'||typeof link.href!=='string'||typeof link.internal!=='boolean')invalid();}
  outcome.result={Ok:value};return outcome;
}
