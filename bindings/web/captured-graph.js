// Private physical storage primitive. No facade, traversal, native ABI or selection.
// Scalar tags match PELGBIN1. Number words never pass through a Float64Array.
export const CAPTURED_PAGE_SIZE = 1024;
const DIRECTORY_SIZE = 256, PAGE_COUNT = DIRECTORY_SIZE * DIRECTORY_SIZE;
const ADDRESS_LIMIT = PAGE_COUNT * CAPTURED_PAGE_SIZE;
const NODE_LIMIT = 1_000_000, STRING_LIMIT = 1_000_000, UNIT_LIMIT = 16_000_000;
const PENDING = 7, ABSENT = 0xffffffff;
const PROPERTY_ATOM = 1, ITEM_ATOM = 1 + ADDRESS_LIMIT, ARGUMENT_ATOM = 1 + 2 * ADDRESS_LIMIT;
export const CapturedKind = Object.freeze({object: 0, instance: 1, array: 2, map: 3, set: 4, date: 5, error: 6});
export const CapturedTag = Object.freeze({undefined: 0, null: 1, false: 2, true: 3, number: 4, string: 5, ref: 6, builtinFunction: 7, hostReference: 8});
const hostBudget = () => { throw new RangeError('Public model graph exceeds host exchange budget'); };
const requireValue = (condition, message) => { if (!condition) throw new TypeError(message); };
const u32 = value => Number.isInteger(value) && value >= 0 && value <= 0xffffffff;
const index = (value, count) => requireValue(Number.isInteger(value) && value >= 0 && value < count, 'Captured storage index out of range');
const capacity = count => { let size = 1; while (size < count) size *= 2; return count ? size : 0; };
function checkAtom(tag, lo, hi) {
  requireValue(Number.isInteger(tag) && tag >= 0 && tag <= 8 && u32(lo) && u32(hi), 'Invalid captured atom words');
  requireValue(tag === 4 || hi === 0, 'Unexpected captured atom high word');
  requireValue(tag >= 4 || lo === 0, 'Unexpected captured atom payload');
}
const atomBytes = tag => tag === 4 ? 9 : tag >= 5 ? 5 : 1;

// A root owns its top directory. Fork rotates BOTH ownership tokens, so subsequent
// writes on either branch clone a shared second-level directory and touched page.
class Columns {
  #types; #top; #owner;
  constructor(types) { this.#types = types; this.#top = new Array(DIRECTORY_SIZE); this.#owner = {}; }
  fork() {
    const next = new Columns(this.#types);
    next.#top = this.#top.slice();
    this.#owner = {}; // Pages written before fork belong to neither branch now.
    return next;
  }
  #page(at, writing) {
    requireValue(Number.isSafeInteger(at) && at >= 0 && at < ADDRESS_LIMIT, 'Captured storage address out of range');
    const page = Math.floor(at / CAPTURED_PAGE_SIZE), high = Math.floor(page / DIRECTORY_SIZE), low = page % DIRECTORY_SIZE;
    let directory = this.#top[high];
    if (!writing) { requireValue(!!directory?.pages[low], 'Missing captured storage page'); return directory.pages[low]; }
    if (!directory || directory.owner !== this.#owner) {
      directory = {owner: this.#owner, pages: directory ? directory.pages.slice() : new Array(DIRECTORY_SIZE)};
      this.#top[high] = directory;
    }
    let data = directory.pages[low];
    if (!data || data.owner !== this.#owner) {
      data = {owner: this.#owner, columns: data ? data.columns.map(column => column.slice()) : this.#types.map(Type => new Type(CAPTURED_PAGE_SIZE))};
      directory.pages[low] = data;
    }
    return data;
  }
  has(at) {
    if (!Number.isSafeInteger(at) || at < 0 || at >= ADDRESS_LIMIT) return false;
    const page = Math.floor(at / CAPTURED_PAGE_SIZE);
    return !!this.#top[Math.floor(page / DIRECTORY_SIZE)]?.pages[page % DIRECTORY_SIZE];
  }
  get(at, column) { return this.#page(at, false).columns[column][at % CAPTURED_PAGE_SIZE]; }
  set(at, column, value) { this.#page(at, true).columns[column][at % CAPTURED_PAGE_SIZE] = value; }
  copy(from, source, to, count) {
    for (let i = 0; i < count; i++) {
      const a = source.#page(from + i, false), b = this.#page(to + i, true);
      const ai = (from + i) % CAPTURED_PAGE_SIZE, bi = (to + i) % CAPTURED_PAGE_SIZE;
      for (let c = 0; c < this.#types.length; c++) b.columns[c][bi] = a.columns[c][ai];
    }
  }
  stats() {
    let directories = 0, pages = 0;
    for (const directory of this.#top) if (directory) { directories++; for (const page of directory.pages) if (page) pages++; }
    return {directories, pages};
  }
}

// Hashes only choose a bucket; exact JS string equality in Map decides identity.
// Each epoch points directly at bucket maps. No overlay/parent chain, no whole
// string-history index clone on fork. Duplicate string IDs remain explicit.
class StringIndexes {
  #buckets = new Array(DIRECTORY_SIZE); #owner = {};
  fork() { const next = new StringIndexes(); next.#buckets = this.#buckets.slice(); this.#owner = {}; return next; }
  #slot(text) {
    let hash = 2166136261;
    for (let i = 0; i < text.length; i++) hash = Math.imul(hash ^ text.charCodeAt(i), 16777619);
    return hash & (DIRECTORY_SIZE - 1);
  }
  find(text, last) {
    requireValue(typeof text === 'string', 'Captured string key must be a string');
    const bucket = this.#buckets[this.#slot(text)];
    return bucket ? (last ? bucket.last : bucket.first).get(text) : undefined;
  }
  append(text, id) {
    const slot = this.#slot(text); let bucket = this.#buckets[slot];
    if (!bucket || bucket.owner !== this.#owner) {
      bucket = {owner: this.#owner, first: new Map(bucket?.first), last: new Map(bucket?.last)};
      this.#buckets[slot] = bucket;
    }
    if (!bucket.first.has(text)) bucket.first.set(text, id);
    bucket.last.set(text, id);
  }
}

export class FlatStrings {
  #table = new Columns([Array]); #indexes = new StringIndexes(); #count = 0; #units = 0;
  #deferModelLimits;
  constructor({deferModelLimits = false} = {}) {
    requireValue(typeof deferModelLimits === 'boolean', 'Invalid captured storage policy');
    this.#deferModelLimits = deferModelLimits;
  }
  get defersModelLimits() { return this.#deferModelLimits; }
  get count() { return this.#count; }
  get units() { return this.#units; }
  fork(deferModelLimits = this.#deferModelLimits) {
    const next = new FlatStrings({deferModelLimits}); next.#table = this.#table.fork(); next.#indexes = this.#indexes.fork();
    next.#count = this.#count; next.#units = this.#units; return next;
  }
  get(id) { index(id, this.#count); return this.#table.get(id, 0); }
  firstId(text) { return this.#indexes.find(text, false); }
  lastId(text) { return this.#indexes.find(text, true); }
  prepareIndexes() { return this; } // Both indexes are always maintained eagerly.
  append(text) {
    requireValue(typeof text === 'string', 'Captured string value must be a string');
    requireValue(Number.isSafeInteger(this.#units + text.length), 'Captured string unit count overflow');
    if (!this.#deferModelLimits && (this.#count >= STRING_LIMIT || this.#units + text.length > UNIT_LIMIT)) hostBudget();
    const id = this.#count;
    this.#table.set(id, 0, text); this.#indexes.append(text, id);
    this.#count++; this.#units += text.length; return id;
  }
  internFirst(text) { const known = this.firstId(text); return known === undefined ? this.append(text) : known; }
  observeLast(text) { const known = this.lastId(text); return known === undefined ? this.append(text) : known; }
  stats() { return {...this.#table.stats(), strings: this.#count, units: this.#units}; }
}

// Missing committed IDs are possible after a failed observation discovers owner
// objects. Zero-filled headers must never turn those holes into ordinary Objects.
class HeaderKinds extends Uint32Array {
  constructor(length) { super(length); this.fill(ABSENT); }
}
const headers = () => new Columns([HeaderKinds, Uint32Array, Uint32Array, Uint32Array, Uint32Array, Uint32Array, Uint32Array]);
const properties = () => new Columns([Uint32Array, Uint8Array, Uint8Array, Uint32Array, Uint32Array]);
const items = () => new Columns([Uint8Array, Uint32Array, Uint32Array]);

export class FlatGraph {
  #headers = headers(); #properties = properties(); #items = items();
  #strings; #deferModelLimits; #nodes = 0; #propEnd = 0; #itemEnd = 0; #props = 0; #atoms = 0;
  #tag = 0; #lo = 0; #hi = 0; #building = -1; #dirty = false;
  #arguments = null; #argumentCount = 0;
  constructor(strings = new FlatStrings()) {
    requireValue(strings instanceof FlatStrings, 'Captured strings require a FlatStrings history');
    this.#deferModelLimits = strings.defersModelLimits; this.#strings = strings.fork();
  }
  get strings() { return this.#strings; }
  get nodeCount() { return this.#nodes; }
  get stringCount() { return this.#strings.count; }
  get stringUnits() { return this.#strings.units; }
  get work() { return this.#props + this.#atoms; }
  stringValueAt(id) { return this.#strings.get(id); }
  fork({deferModelLimits = this.#deferModelLimits} = {}) {
    requireValue(typeof deferModelLimits === 'boolean', 'Invalid captured storage policy');
    requireValue(this.#building === -1, 'Cannot fork unfinished captured node');
    const next = new FlatGraph();
    next.#headers = this.#headers.fork(); next.#properties = this.#properties.fork(); next.#items = this.#items.fork(); next.#strings = this.#strings.fork(deferModelLimits); next.#deferModelLimits = deferModelLimits;
    next.#nodes = this.#nodes; next.#propEnd = this.#propEnd; next.#itemEnd = this.#itemEnd; next.#props = this.#props; next.#atoms = this.#atoms;
    next.#tag = this.#tag; next.#lo = this.#lo; next.#hi = this.#hi; next.#dirty = this.#dirty;
    return next;
  }
  // Attach the independently retained observation history by forking its root.
  // The caller retains graph.strings if string additions must outlive failed W.
  adoptStrings(history) { requireValue(history instanceof FlatStrings, 'Invalid captured string history'); this.#strings = history.fork(this.#deferModelLimits); }
  setRoot(tag, lo = 0, hi = 0) { checkAtom(tag, lo, hi); this.#tag = tag; this.#lo = lo; this.#hi = hi; }
  rootAtom(out, offset = 0) { out[offset] = this.#tag; out[offset + 1] = this.#lo; out[offset + 2] = this.#hi; return out; }
  #kind(id) { return this.#headers.has(id) ? this.#headers.get(id, 0) : ABSENT; }
  #checkNode(id) {
    requireValue(Number.isInteger(id) && id >= 0 && (id < this.#nodes || id === this.#building) && this.#kind(id) !== ABSENT, 'Missing captured node');
  }
  header(id, out, offset = 0) {
    index(id, this.#nodes); if (this.#kind(id) === ABSENT) return false;
    requireValue(this.#headers.get(id, 0) !== PENDING, 'Unfinished captured node is not readable');
    for (let c = 0; c < 7; c++) out[offset + c] = this.#headers.get(id, c);
    return out;
  }
  // Direct scalar reader seam. No header/property scratch refill per field.
  kindCode(id) { index(id, this.#nodes); return this.#kind(id); }
  headerWord(id, column) { this.#checkNode(id); index(column, 7); return this.#headers.get(id, column); }
  propertyAddress(id, slot) { this.#checkNode(id); index(slot, this.#headers.get(id, 4)); return this.#headers.get(id, 3) + slot; }
  propertyWord(at, column) { index(at, this.#propEnd); index(column, 5); return this.#properties.get(at, column); }
  itemAddress(id, slot) { this.#checkNode(id); index(slot, this.#itemCount(id)); return this.#headers.get(id, 5) + slot; }
  atomWord(slot, column) {
    requireValue(Number.isSafeInteger(slot) && slot >= 0, 'Invalid captured atom slot'); index(column, 3);
    if (slot === 0) return column === 0 ? this.#tag : column === 1 ? this.#lo : this.#hi;
    if (slot < ITEM_ATOM) return this.propertyWord(slot - PROPERTY_ATOM, column + 2);
    if (slot < ARGUMENT_ATOM) { const at = slot - ITEM_ATOM; index(at, this.#itemEnd); return this.#items.get(at, column); }
    const at = slot - ARGUMENT_ATOM; index(at, this.#argumentCount); return this.#arguments.get(at, column);
  }
  beginArguments() { this.#arguments = items(); this.#argumentCount = 0; }
  appendArgument(tag, lo, hi) {
    requireValue(this.#arguments !== null, 'Captured arguments are not open'); checkAtom(tag, lo, hi);
    const at = this.#argumentCount;
    requireValue(at < ADDRESS_LIMIT, 'Captured physical argument address space exhausted');
    this.#writeAtom(this.#arguments, at, 0, tag, lo, hi); this.#argumentCount++;
    return ARGUMENT_ATOM + at;
  }
  releaseArguments() { this.#arguments = null; this.#argumentCount = 0; }
  property(id, slot, out, offset = 0) {
    this.#checkNode(id); index(slot, this.#headers.get(id, 4));
    const at = this.#headers.get(id, 3) + slot;
    for (let c = 0; c < 5; c++) out[offset + c] = this.#properties.get(at, c);
    return out;
  }
  #itemCount(id) { const kind = this.#headers.get(id, 0), count = this.#headers.get(id, 6); return kind === 3 ? 2 * count : count; }
  item(id, atomSlot, out, offset = 0) {
    this.#checkNode(id); index(atomSlot, this.#itemCount(id)); const at = this.#headers.get(id, 5) + atomSlot;
    for (let c = 0; c < 3; c++) out[offset + c] = this.#items.get(at, c);
    return out;
  }
  beginNode(id) {
    requireValue(this.#building === -1, 'A captured node is already open');
    requireValue(Number.isInteger(id) && id >= 0, 'Invalid captured node ID');
    if (!this.#deferModelLimits && id >= NODE_LIMIT) hostBudget();
    if (this.#kind(id) !== ABSENT) { this.#props -= this.#headers.get(id, 4); this.#atoms -= this.#itemCount(id); }
    for (let c = 0; c < 7; c++) this.#headers.set(id, c, c === 0 ? PENDING : 0);
    this.#building = id; this.#dirty = true;
  }
  finishNode(id, kind, aux0 = 0, aux1 = 0) {
    requireValue(id === this.#building, 'Captured node completion is out of order');
    requireValue(Number.isInteger(kind) && kind >= 0 && kind <= 6 && u32(aux0) && u32(aux1), 'Invalid captured node header');
    const count = this.#headers.get(id, 6);
    requireValue(kind === 3 ? count % 2 === 0 : kind === 4 || count === 0, 'Captured payload does not match node kind');
    this.#headers.set(id, 0, kind); this.#headers.set(id, 1, aux0); this.#headers.set(id, 2, aux1);
    if (kind === 3) this.#headers.set(id, 6, count / 2);
    this.#nodes = Math.max(this.#nodes, id + 1); this.#building = -1;
  }
  setHeaderAux(id, aux0, aux1 = 0) {
    this.#checkNode(id); requireValue(u32(aux0) && u32(aux1), 'Invalid captured header words');
    this.#headers.set(id, 1, aux0); this.#headers.set(id, 2, aux1);
  }
  #reserve(which, count) {
    requireValue(Number.isSafeInteger(count) && count > 0 && count <= ADDRESS_LIMIT, 'Invalid captured span capacity');
    let end = which === 0 ? this.#propEnd : this.#itemEnd;
    if (end + count > ADDRESS_LIMIT) { this.compact(); end = which === 0 ? this.#propEnd : this.#itemEnd; }
    // After compaction, valid public graphs (<=8M live slots) fit this >67M
    // address space. No smaller public budget or dead-span charge is introduced.
    requireValue(Number.isSafeInteger(end + count) && end + count <= ADDRESS_LIMIT && end + count <= 0xffffffff, 'Captured physical address space exhausted');
    if (which === 0) this.#propEnd = end + count; else this.#itemEnd = end + count;
    return end;
  }
  #grow(id, which, oldCount, add) {
    const field = which === 0 ? 3 : 5, next = oldCount + add;
    requireValue(Number.isSafeInteger(next), 'Captured span length overflow');
    if (next <= capacity(oldCount)) return this.#headers.get(id, field);
    const start = this.#reserve(which, capacity(next));
    // Reservation may compact: reload BOTH old start and table afterwards.
    const oldStart = this.#headers.get(id, field), table = which === 0 ? this.#properties : this.#items;
    if (oldCount) table.copy(oldStart, table, start, oldCount);
    this.#headers.set(id, field, start); return start;
  }
  #writeAtom(table, at, column, tag, lo, hi) { table.set(at, column, tag); table.set(at, column + 1, lo); table.set(at, column + 2, hi); }
  appendProperty(id, key, flags, tag, lo = 0, hi = 0) {
    this.#checkNode(id); requireValue(u32(key) && Number.isInteger(flags) && flags >= 0 && flags <= 7, 'Invalid captured property'); checkAtom(tag, lo, hi);
    const count = this.#headers.get(id, 4), start = this.#grow(id, 0, count, 1), at = start + count;
    this.#properties.set(at, 0, key); this.#properties.set(at, 1, flags); this.#writeAtom(this.#properties, at, 2, tag, lo, hi);
    this.#headers.set(id, 4, count + 1); this.#props++; this.#dirty = true;
  }
  // Observer/constructor payloads arrive one atom at a time while kind is pending.
  // Completed Maps use appendMapEntry so no exposed half-pair can exist.
  appendItem(id, tag, lo = 0, hi = 0) {
    this.#checkNode(id); const kind = this.#headers.get(id, 0);
    requireValue(kind === PENDING || kind === 4, 'Single item append requires pending node or Set'); checkAtom(tag, lo, hi);
    const count = this.#headers.get(id, 6), start = this.#grow(id, 1, count, 1);
    this.#writeAtom(this.#items, start + count, 0, tag, lo, hi); this.#headers.set(id, 6, count + 1); this.#atoms++; this.#dirty = true;
  }
  appendMapEntry(id, keyTag, keyLo, keyHi, valueTag, valueLo, valueHi) {
    this.#checkNode(id); requireValue(this.#headers.get(id, 0) === 3, 'Captured node is not a Map'); checkAtom(keyTag, keyLo, keyHi); checkAtom(valueTag, valueLo, valueHi);
    const pairs = this.#headers.get(id, 6), count = pairs * 2, start = this.#grow(id, 1, count, 2);
    this.#writeAtom(this.#items, start + count, 0, keyTag, keyLo, keyHi); this.#writeAtom(this.#items, start + count + 1, 0, valueTag, valueLo, valueHi);
    this.#headers.set(id, 6, pairs + 1); this.#atoms += 2; this.#dirty = true;
  }
  replacePropertyValue(id, slot, tag, lo = 0, hi = 0) {
    this.#checkNode(id); index(slot, this.#headers.get(id, 4)); checkAtom(tag, lo, hi);
    this.#writeAtom(this.#properties, this.#headers.get(id, 3) + slot, 2, tag, lo, hi);
  }
  replaceItem(id, slot, tag, lo = 0, hi = 0) {
    this.#checkNode(id); index(slot, this.#itemCount(id)); checkAtom(tag, lo, hi);
    this.#writeAtom(this.#items, this.#headers.get(id, 5) + slot, 0, tag, lo, hi);
  }
  compact() {
    // Construct new tables and headers transactionally; no model reads/callbacks.
    const nextHeaders = this.#headers.fork(), nextProps = properties(), nextItems = items();
    let p = 0, a = 0;
    for (let id = 0; id < Math.max(this.#nodes, this.#building + 1); id++) {
      if (this.#kind(id) === ABSENT) continue;
      const pc = this.#headers.get(id, 4), ac = this.#itemCount(id);
      if (pc) { nextProps.copy(this.#headers.get(id, 3), this.#properties, p, pc); nextHeaders.set(id, 3, p); p += capacity(pc); }
      else nextHeaders.set(id, 3, 0);
      if (ac) { nextItems.copy(this.#headers.get(id, 5), this.#items, a, ac); nextHeaders.set(id, 5, a); a += capacity(ac); }
      else nextHeaders.set(id, 5, 0);
    }
    this.#headers = nextHeaders; this.#properties = nextProps; this.#items = nextItems; this.#propEnd = p; this.#itemEnd = a; this.#dirty = false;
  }
  finishBoundary() {
    requireValue(this.#building === -1, 'Unfinished captured node at operation boundary');
    if (this.#dirty && (this.#propEnd > 4 * this.#props + 2 * CAPTURED_PAGE_SIZE || this.#itemEnd > 4 * this.#atoms + 2 * CAPTURED_PAGE_SIZE)) this.compact();
    this.#dirty = false; this.#strings.prepareIndexes(); return this;
  }
  // Exact size only. The caller performs the legacy size gate BEFORE scalar
  // validity, and its own graph/schema/reference admission at the original phase.
  encodedGraphBytes(metadataBytes = 0) {
    requireValue(this.#building === -1 && Number.isSafeInteger(metadataBytes) && metadataBytes >= 0, 'Invalid captured frame sizing request');
    let bytes = 32 + metadataBytes + atomBytes(this.#tag) + 4 * this.#strings.count + 2 * this.#strings.units;
    for (let id = 0; id < this.#nodes; id++) {
      requireValue(this.#kind(id) !== ABSENT, 'Missing captured node during frame sizing');
      const kind = this.#headers.get(id, 0), pc = this.#headers.get(id, 4), ac = this.#itemCount(id);
      bytes += 5 + (kind === 6 ? 1 : kind === 5 ? 8 : kind === 1 || kind === 2 || kind === 3 || kind === 4 ? 4 : 0);
      const ps = this.#headers.get(id, 3), as = this.#headers.get(id, 5);
      for (let i = 0; i < pc; i++) bytes += 5 + atomBytes(this.#properties.get(ps + i, 2));
      for (let i = 0; i < ac; i++) bytes += atomBytes(this.#items.get(as + i, 0));
    }
    return bytes;
  }
  firstInvalidScalarNode() {
    requireValue(this.#building === -1, 'Unfinished captured node at scalar admission');
    for (let id = 0; id < this.#nodes; id++) { const kind = this.#kind(id); if (kind === ABSENT) return id; if ((kind === 2 || kind === 6) && this.#headers.get(id, 2) !== 0) return id; }
    return undefined;
  }
  stats() {
    return {nodes: this.#nodes, strings: this.#strings.stats(), headers: this.#headers.stats(), properties: {...this.#properties.stats(), allocatedSlots: this.#propEnd, reachableSlots: this.#props}, items: {...this.#items.stats(), allocatedSlots: this.#itemEnd, reachableSlots: this.#atoms}, arguments: this.#arguments ? {...this.#arguments.stats(), slots: this.#argumentCount} : {directories: 0, pages: 0, slots: 0}, unfinishedNode: this.#building};
  }
}

const kindNames = Object.freeze(['object', 'instance', 'array', 'map', 'set', 'date', 'error']);
const errorNames = Object.freeze(['error', 'pele', 'type', 'range', 'syntax', 'reference', 'eval', 'uri']);
const hexWords = (lo, hi) => hi.toString(16).padStart(8, '0') + lo.toString(16).padStart(8, '0');
// Deliberately use the same missing-node property access as the nested reader.
// Engine-native TypeError text is preserved without inventing a message string.
function missingNodeKind() { const absent = undefined; return absent.kind; }
const publicationBits = new DataView(new ArrayBuffer(8));

// Bound only by the owner which maintains graph/table identity together. The
// supplied Array is internal observation history, never caller-owned model data.
// Binding asserts the already-established prefix relation; it does not rescan it.
// Full reconcile must bind its replacement graph and replacement table together.
const observedTables = new WeakMap();
export function bindCapturedStringTable(graph, table) {
  requireValue(graph instanceof FlatGraph && Array.isArray(table) && table.length >= graph.stringCount, 'Invalid captured observation table binding');
  observedTables.set(graph, table);
}

class FlatCapture {
  #table;
  #graph; #values = null; #node = -1; #itemsKind = null;
  #bits = new DataView(new ArrayBuffer(8)); #lo = 0; #hi = 0;
  constructor(baseline, delta) {
    requireValue(baseline instanceof FlatGraph, 'Flat capture requires a FlatGraph baseline');
    requireValue(delta === false, 'Flat delta transport is not implemented');
    this.#table = observedTables.get(baseline);
    requireValue(this.#table !== undefined && this.#table.length >= baseline.stringCount, 'Captured observation table is not bound');
    this.#graph = baseline.fork({deferModelLimits: false});
  }
  #open() { requireValue(this.#graph !== null, 'Flat capture is closed'); return this.#graph; }
  #decode(tag, payload) {
    if (tag === 4) {
      requireValue(typeof payload === 'number', 'Captured numeric payload must be a number');
      this.#bits.setFloat64(0, payload, true);
      this.#lo = this.#bits.getUint32(0, true); this.#hi = this.#bits.getUint32(4, true);
    } else { this.#lo = tag >= 5 ? payload : 0; this.#hi = 0; }
    checkAtom(tag, this.#lo, this.#hi);
  }
  root(tag, payload) { const graph = this.#open(); this.#decode(tag, payload); graph.setRoot(tag, this.#lo, this.#hi); }
  beginArguments(length) {
    const graph = this.#open(); requireValue(u32(length), 'Invalid captured argument length');
    graph.beginArguments(); this.#values = new Array(length);
  }
  argument(at, tag, payload) {
    const graph = this.#open(); requireValue(this.#values !== null, 'Captured arguments are not open'); index(at, this.#values.length);
    this.#decode(tag, payload); this.#values[at] = graph.appendArgument(tag, this.#lo, this.#hi);
  }
  beginNode(id) { this.#open().beginNode(id); this.#node = id; this.#itemsKind = null; }
  property(key, flags, tag, payload) {
    const graph = this.#open(); requireValue(this.#node !== -1, 'No captured node is open'); this.#decode(tag, payload);
    graph.appendProperty(this.#node, key, flags, tag, this.#lo, this.#hi);
  }
  beginItems(kind) {
    this.#open(); requireValue(this.#node !== -1 && (kind === 'map' || kind === 'set'), 'Invalid captured collection');
    this.#itemsKind = kind;
  }
  mapEntry(keyTag, keyPayload, valueTag, valuePayload) {
    const graph = this.#open(); requireValue(this.#itemsKind === 'map', 'Captured Map items are not open');
    this.#decode(keyTag, keyPayload); const keyLo = this.#lo, keyHi = this.#hi;
    this.#decode(valueTag, valuePayload);
    graph.appendItem(this.#node, keyTag, keyLo, keyHi); graph.appendItem(this.#node, valueTag, this.#lo, this.#hi);
  }
  setItem(tag, payload) {
    const graph = this.#open(); requireValue(this.#itemsKind === 'set', 'Captured Set items are not open'); this.#decode(tag, payload);
    graph.appendItem(this.#node, tag, this.#lo, this.#hi);
  }
  finishNode(id, kind, aux) {
    const graph = this.#open(); requireValue(this.#node === id, 'Captured node completion is out of order');
    const code = kindNames.indexOf(kind); requireValue(code >= 0, 'Unknown captured node kind');
    let lo = 0, hi = 0;
    if (kind === 'date') {
      requireValue(typeof aux === 'string' && /^[0-9a-fA-F]{16}$/.test(aux), 'Invalid captured Date bits');
      lo = Number.parseInt(aux.slice(8), 16); hi = Number.parseInt(aux.slice(0, 8), 16);
    } else if (kind === 'array') { if (u32(aux)) lo = aux; else hi = 1; }
    else if (kind === 'error') { const brand = errorNames.indexOf(aux); if (brand >= 0) lo = brand; else hi = 1; }
    else if (kind === 'instance') { requireValue(u32(aux), 'Invalid captured prototype ID'); lo = aux; }
    graph.finishNode(id, code, lo, hi); this.#node = -1; this.#itemsKind = null;
  }
  finish(strings) {
    const graph = this.#open(); requireValue(this.#node === -1 && this.#values !== null, 'Incomplete flat capture');
    // Failed observations append to the same owner table. Its prefix is an owner
    // invariant, bound explicitly above; only the newly observed suffix is copied.
    requireValue(strings === this.#table && strings.length >= graph.stringCount, 'Captured observation table relation changed');
    for (let i = graph.stringCount; i < strings.length; i++) graph.strings.append(strings[i]);
    observedTables.set(graph, strings);
    graph.finishBoundary(); const values = this.#values;
    this.#graph = null; this.#values = null; this.#table = null; return {graph, values};
  }
  abort() {
    this.#graph?.releaseArguments(); this.#graph = null; this.#values = null; this.#table = null; this.#node = -1; this.#itemsKind = null;
  }
}

// Implements the scalar seam in graph-storage.js; no selector imports this yet.
// Property iterators yield physical offsets. Atom slots use disjoint numeric
// domains, not wrapper objects. Slots are scoped to one immutable read phase;
// compaction/write invalidates old property/item addresses.
export const FlatStorage = Object.freeze({
  stringCount: graph => graph.stringCount,
  stringValueAt: (graph, id) => graph.stringValueAt(id),
  nodeCount: graph => graph.nodeCount,
  *nodeIds(graph) { for (let id = 0; id < graph.nodeCount; id++) yield id; },
  rootSlot: () => 0,
  nodeKind(graph, id) { const code = graph.kindCode(id); return code === ABSENT ? missingNodeKind() : kindNames[code]; },
  nodeAux(graph, id) {
    const kind = graph.kindCode(id); if (kind === ABSENT) return missingNodeKind();
    const lo = graph.headerWord(id, 1), hi = graph.headerWord(id, 2);
    if (kind === 5) return hexWords(lo, hi);
    if (kind === 6) return hi ? undefined : errorNames[lo];
    return kind === 2 && hi ? undefined : lo;
  },
  propertyCount: (graph, id) => graph.headerWord(id, 4),
  propertySlot: (graph, id, at) => graph.propertyAddress(id, at),
  *propertySlots(graph, id) {
    const start = graph.headerWord(id, 3), count = graph.headerWord(id, 4);
    for (let at = 0; at < count; at++) yield start + at;
  },
  propertyKey: (graph, slot) => graph.propertyWord(slot, 0),
  propertyFlags: (graph, slot) => graph.propertyWord(slot, 1),
  propertyAtomSlot: (_graph, slot) => PROPERTY_ATOM + slot,
  itemCount: (graph, id) => graph.headerWord(id, 6),
  itemAtomSlot: (graph, id, at) => ITEM_ATOM + graph.itemAddress(id, at),
  atomTag: (graph, slot) => graph.atomWord(slot, 0),
  atomPayload(graph, slot) {
    const tag = graph.atomWord(slot, 0), lo = graph.atomWord(slot, 1);
    if (tag !== 4) return lo;
    publicationBits.setUint32(0, lo, true); publicationBits.setUint32(4, graph.atomWord(slot, 2), true);
    return publicationBits.getFloat64(0, true);
  },
  atomNumberBits: (graph, slot) => hexWords(graph.atomWord(slot, 1), graph.atomWord(slot, 2)),
  beginCapture: (baseline, delta = false) => new FlatCapture(baseline, delta),
  releaseArguments: graph => graph.releaseArguments(),
});
