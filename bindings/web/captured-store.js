// Private captured-column ABI. No facade reads, diagram policy or activation.
import {FlatGraph, FlatStrings, FlatStorage} from './captured-graph.js';
import {PeleError} from './errors.js';
import {FlatConstruction, FlatAppendConstruction} from './flat-constructor.js';

const ABSENT = 0xffffffff;
const contexts = new WeakMap();
const u32 = value => Number.isInteger(value) && value >= 0 && value <= ABSENT;
function protocol(condition, message = 'Invalid captured storage request') {
  if (!condition) throw new TypeError(message);
}
function semantic(message) { throw new PeleError(message, 'semantic'); }
function state(context) {
  const value = contexts.get(context);
  protocol(value !== undefined, 'Captured storage context is closed');
  return value;
}
// Internal key strings are copied once, outside the model string history. Writable
// contexts always fork: even a mistakenly supplied accepted epoch is never edited.
export function createCapturedContext(owner, epoch, queryKeys, writable = false) {
  protocol(epoch instanceof FlatGraph && Array.isArray(queryKeys) && typeof writable === 'boolean');
  const keys = new Array(queryKeys.length);
  for (let i = 0; i < keys.length; i++) { protocol(typeof queryKeys[i] === 'string'); keys[i] = queryKeys[i]; }
  const context = Object.freeze({}), queryIndex = new Map();
  for (let i = 0; i < keys.length; i++) if (!queryIndex.has(keys[i])) queryIndex.set(keys[i], i);
  contexts.set(context, {owner, epoch: writable ? epoch.fork({deferModelLimits: true}) : epoch, queryKeys: keys, queryIndex, queryFragment: null, writable});
  return context;
}
// Epoch ownership/admission/commit remain the caller's responsibility. Extract W
// before closing when partial helper writes must survive an error.
export function capturedEpoch(context) { return state(context).epoch; }
export function closeCapturedContext(context) { contexts.delete(context); }
function hasNode(graph, id) {
  protocol(u32(id));
  if (id >= graph.nodeCount || graph.kindCode(id) === ABSENT) return false;
  protocol(graph.kindCode(id) <= 6, 'Unfinished captured node is not readable');
  return true;
}
function node(graph, id, kind) {
  if (!hasNode(graph, id)) semantic('Invalid captured object reference');
  if (kind !== undefined) protocol(graph.kindCode(id) === kind, 'Captured node kind does not match request');
}
function text(graph, id) {
  protocol(u32(id));
  if (id >= graph.stringCount) semantic('Invalid public string reference');
  return graph.stringValueAt(id);
}
function windowCount(start, count, length, max, message = 'Invalid captured cursor') {
  protocol(u32(start) && u32(count) && count <= max);
  if (start > length) semantic(message);
  return Math.min(count, length - start);
}
function keysFor(value, handles) {
  protocol(handles instanceof Uint32Array && handles.length <= 128);
  for (let i = 0; i < handles.length; i++) protocol(handles[i] < value.queryKeys.length, 'Invalid captured query key');
  return value.queryKeys;
}
function copyAtom(graph, slot, out, at) {
  out[at] = graph.atomWord(slot, 0); out[at + 1] = graph.atomWord(slot, 1); out[at + 2] = graph.atomWord(slot, 2);
}
function itemAtom(graph, id, slot) { return FlatStorage.itemAtomSlot(graph, id, slot); }
function validAtom(tag, lo, hi) {
  protocol(u32(tag) && tag <= 8 && u32(lo) && u32(hi) && (tag === 4 || hi === 0) && (tag >= 4 || lo === 0), 'Invalid captured atom words');
}
export function capturedRoot(context) { return state(context).epoch.rootAtom(new Uint32Array(3)); }
export function capturedHeader(context, id) {
  const graph = state(context).epoch;
  return hasNode(graph, id) ? graph.header(id, new Uint32Array(7)) : new Uint32Array(0);
}
export function capturedNodeHeaders(context, start, count) {
  const graph = state(context).epoch, size = windowCount(start, count, graph.nodeCount, 64);
  const out = new Uint32Array(size * 7);
  for (let i = 0; i < size; i++) {
    if (hasNode(graph, start + i)) graph.header(start + i, out, i * 7);
    else out[i * 7] = ABSENT;
  }
  return out;
}
export function capturedOwnFields(context, id, handles) {
  const value = state(context), graph = value.epoch, keys = keysFor(value, handles), out = new Uint32Array(handles.length * 4);
  if (!hasNode(graph, id) || handles.length === 0) return out;
  let remaining = handles.length;
  for (let i = 0, count = graph.headerWord(id, 4); i < count; i++) {
    const at = graph.propertyAddress(id, i), key = text(graph, graph.propertyWord(at, 0));
    for (let j = 0; j < handles.length; j++) if (!out[j * 4] && key === keys[handles[j]]) {
      out[j * 4] = 1; copyAtom(graph, FlatStorage.propertyAtomSlot(graph, at), out, j * 4 + 1); remaining--;
    }
    if (remaining === 0) break;
  }
  return out;
}
export function capturedOwnDescriptors(context, id, handles) {
  const value = state(context), graph = value.epoch, keys = keysFor(value, handles), out = new Uint32Array(handles.length * 3);
  if (!hasNode(graph, id) || handles.length === 0) return out;
  for (let i = 0, count = graph.headerWord(id, 4); i < count; i++) {
    const at = graph.propertyAddress(id, i), key = text(graph, graph.propertyWord(at, 0));
    for (let j = 0; j < handles.length; j++) if (key === keys[handles[j]]) {
      if (!out[j * 3]) { out[j * 3] = 1; out[j * 3 + 1] = i; }
      if ((graph.propertyWord(at, 1) & 2) === 0) out[j * 3 + 2] = 1;
    }
  }
  return out;
}
export function capturedPropertyPrefixes(context, id, start, count) {
  const graph = state(context).epoch; node(graph, id);
  const size = windowCount(start, count, graph.headerWord(id, 4), 128, 'Invalid captured property cursor'), out = new Uint32Array(size * 9);
  for (let i = 0; i < size; i++) {
    const at = graph.propertyAddress(id, start + i), key = text(graph, graph.propertyWord(at, 0)), offset = i * 9;
    out[offset] = key.length;
    for (let j = 0; j < Math.min(key.length, 10); j++) out[offset + 1 + (j >> 1)] |= key.charCodeAt(j) << ((j & 1) * 16);
    copyAtom(graph, FlatStorage.propertyAtomSlot(graph, at), out, offset + 6);
  }
  return out;
}
export function capturedStringLength(context, id) { return text(state(context).epoch, id).length; }
export function capturedStringUnits(context, id, start, count) {
  const value = text(state(context).epoch, id), size = windowCount(start, count, value.length, 1024), out = new Uint16Array(size);
  for (let i = 0; i < size; i++) out[i] = value.charCodeAt(start + i);
  return out;
}
export function capturedStringLengths(context, start, count) {
  const graph = state(context).epoch, size = windowCount(start, count, graph.stringCount, 256), out = new Uint32Array(size);
  for (let i = 0; i < size; i++) out[i] = graph.stringValueAt(start + i).length;
  return out;
}
export function capturedCounts(context) {
  const graph = state(context).epoch;
  return new Uint32Array([graph.nodeCount, graph.stringCount, graph.stringUnits, graph.work]);
}
export function capturedFindStrings(context, handles) {
  const value = state(context), keys = keysFor(value, handles), out = new Uint32Array(handles.length);
  for (let i = 0; i < handles.length; i++) out[i] = value.epoch.strings.firstId(keys[handles[i]]) ?? ABSENT;
  return out;
}
function isNaNWords(lo, hi) { return (hi & 0x7ff00000) === 0x7ff00000 && ((hi & 0xfffff) !== 0 || lo !== 0); }
function equalKey(graph, slot, tag, lo, hi) {
  if (graph.atomWord(slot, 0) !== tag) return false;
  const leftLo = graph.atomWord(slot, 1), leftHi = graph.atomWord(slot, 2);
  if (tag === 4) return (leftLo === lo && leftHi === hi) ||
    (leftLo === 0 && lo === 0 && (leftHi & 0x7fffffff) === 0 && (hi & 0x7fffffff) === 0) ||
    (isNaNWords(leftLo, leftHi) && isNaNWords(lo, hi));
  if (tag === 5) return text(graph, leftLo) === text(graph, lo);
  return leftLo === lo;
}
export function capturedMapFind(context, id, tag, lo, hi) {
  const graph = state(context).epoch; node(graph, id, 3); validAtom(tag, lo, hi);
  const out = new Uint32Array(5);
  for (let i = 0, count = graph.headerWord(id, 6); i < count; i++) if (equalKey(graph, itemAtom(graph, id, 2 * i), tag, lo, hi)) {
    out[0] = 1; out[1] = i; copyAtom(graph, itemAtom(graph, id, 2 * i + 1), out, 2); break;
  }
  return out;
}
// Admission exports preserve physical order, duplicates, raw NaN bits and invalid
// header markers. They do not perform schema/frame validation ahead of its phase.
export function capturedProperties(context, id, start, count) {
  const graph = state(context).epoch; node(graph, id);
  const size = windowCount(start, count, graph.headerWord(id, 4), 128), out = new Uint32Array(size * 5);
  for (let i = 0; i < size; i++) {
    const at = graph.propertyAddress(id, start + i);
    for (let c = 0; c < 5; c++) out[i * 5 + c] = graph.propertyWord(at, c);
  }
  return out;
}
export function capturedMapEntries(context, id, start, count) {
  const graph = state(context).epoch; node(graph, id, 3);
  const size = windowCount(start, count, graph.headerWord(id, 6), 128), out = new Uint32Array(size * 6);
  for (let i = 0; i < size; i++) {
    copyAtom(graph, itemAtom(graph, id, 2 * (start + i)), out, i * 6);
    copyAtom(graph, itemAtom(graph, id, 2 * (start + i) + 1), out, i * 6 + 3);
  }
  return out;
}
export function capturedSetValues(context, id, start, count) {
  const graph = state(context).epoch; node(graph, id, 4);
  const size = windowCount(start, count, graph.headerWord(id, 6), 256), out = new Uint32Array(size * 3);
  for (let i = 0; i < size; i++) copyAtom(graph, itemAtom(graph, id, start + i), out, i * 3);
  return out;
}
export function capturedWriteBatch(context, bytes) {
  const value = state(context), graph = value.epoch;
  protocol(value.writable, 'Captured storage context is read-only');
  protocol(bytes instanceof Uint8Array && bytes.byteLength >= 4 && bytes.byteLength <= 65536);
  const data = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  const count = data.getUint16(0, true); protocol(count <= 128 && data.getUint16(2, true) === 1);
  let at = 4;
  for (let operation = 0; operation < count; operation++) {
    protocol(at + 5 <= data.byteLength, 'Truncated captured write operation');
    const opcode = data.getUint8(at), length = data.getUint32(at + 1, true); at += 5;
    const expected = opcode === 1 || opcode === 3 || opcode === 5 ? 17 : opcode === 2 ? 22 : opcode === 4 ? 18 : opcode === 6 ? 8 : -1;
    protocol(length === expected && at + length <= data.byteLength, 'Invalid captured write operation');
    const id = data.getUint32(at, true); node(graph, id);
    if (opcode === 6) { node(graph, id, 2); graph.setHeaderAux(id, data.getUint32(at + 4, true)); }
    else {
      const slot = data.getUint32(at + 4, true), atomAt = at + (opcode === 4 ? 9 : opcode === 2 ? 4 : 8);
      const tag = data.getUint8(atomAt), lo = data.getUint32(atomAt + 1, true), hi = data.getUint32(atomAt + 5, true);
      validAtom(tag, lo, hi);
      if (opcode === 1) {
        node(graph, id, 3); protocol(slot < graph.headerWord(id, 6)); graph.replaceItem(id, 2 * slot + 1, tag, lo, hi);
      } else if (opcode === 2) {
        node(graph, id, 3);
        const tag2 = data.getUint8(atomAt + 9), lo2 = data.getUint32(atomAt + 10, true), hi2 = data.getUint32(atomAt + 14, true);
        validAtom(tag2, lo2, hi2); graph.appendMapEntry(id, tag, lo, hi, tag2, lo2, hi2);
      } else if (opcode === 3) graph.replacePropertyValue(id, slot, tag, lo, hi);
      else if (opcode === 4) graph.appendProperty(id, slot, data.getUint8(at + 8), tag, lo, hi);
      else {
        protocol(slot < value.queryKeys.length, 'Invalid captured query key');
        for (let i = 0, size = graph.headerWord(id, 4); i < size; i++) {
          const property = graph.propertyAddress(id, i);
          if (text(graph, graph.propertyWord(property, 0)) === value.queryKeys[slot]) graph.replacePropertyValue(id, i, tag, lo, hi);
        }
      }
    }
    at += length;
  }
  protocol(at === data.byteLength, 'Trailing captured write bytes');
  return 0;
}
// AppendNode/Intern below consume the shared constructor decoder; no separate
// node/atom encoding or live model publication is introduced.


// Query registration is separate from canonical model string history. Existing
// handles never change; large single names use bounded raw UTF16 fragments.
const QUERY_UNITS = 32768;
function queryId(value, text) {
  const old = value.queryIndex.get(text);
  if (old !== undefined) return old;
  protocol(value.queryKeys.length < 0xffffffff, 'Captured query handle space exhausted');
  const id = value.queryKeys.length; value.queryKeys.push(text); value.queryIndex.set(text, id); return id;
}
export function capturedQueryKeys(context, names) {
  const value = state(context); protocol(Array.isArray(names) && names.length <= 128);
  let units = 0;
  for (const name of names) { protocol(typeof name === 'string'); units += name.length; protocol(units <= QUERY_UNITS); }
  const out = new Uint32Array(names.length);
  for (let i = 0; i < names.length; i++) out[i] = queryId(value, names[i]);
  return out;
}
export function capturedBeginQueryKey(context, length) {
  const value = state(context); protocol(u32(length) && value.queryFragment === null);
  value.queryFragment = {length, read: 0, parts: []};
}
export function capturedQueryKeyChunk(context, units) {
  const value = state(context), fragment = value.queryFragment;
  protocol(fragment !== null && units instanceof Uint16Array && units.length > 0 && units.length <= 1024 && fragment.read + units.length <= fragment.length);
  fragment.parts.push(String.fromCharCode(...units)); fragment.read += units.length;
}
export function capturedFinishQueryKey(context) {
  const value = state(context), fragment = value.queryFragment;
  protocol(fragment !== null && fragment.read === fragment.length);
  const text = fragment.parts.join(''); value.queryFragment = null; return queryId(value, text);
}
export function createCapturedConstructionContext(owner, prototypeId) {
  protocol(u32(prototypeId));
  const graph = new FlatGraph(new FlatStrings({deferModelLimits: true}));
  const context = createCapturedContext(owner, graph, [], false), value = state(context);
  value.writable = true; value.construction = new FlatConstruction(graph, prototypeId); return context;
}
export function capturedPublishConstruction(context, bytes, prototypeId) {
  const value = state(context); protocol(value.writable && u32(prototypeId));
  if (!value.construction) {
    if (!value.append) value.append = new FlatAppendConstruction(value.epoch, prototypeId);
    protocol(value.append.prototypeId === prototypeId);
  }
  return (value.construction || value.append).publishBatch(bytes);
}
export function capturedFinishAppend(context) {
  const value = state(context); protocol(value.append !== undefined && !value.construction);
  value.append.finishBatch(); return 0;
}
export function capturedFinishConstruction(context, tag, lo, hi) {
  const value = state(context); protocol(value.construction !== undefined); validAtom(tag, lo, hi);
  value.construction.setRoot(tag, lo, hi); value.epoch = value.construction.takeGraph(); delete value.construction; return 0;
}
export function capturedAbortConstruction(context) {
  const value = state(context); protocol(value.construction !== undefined);
  value.construction.discard(); closeCapturedContext(context); return 0;
}
