// Host values stay opaque across resource callbacks. Rust chooses every field
// and render phase; this adapter supplies ordinary JavaScript value operations.
// Typed operands avoid converting the same control strings on every property
// read. The callback itself remains job-local and owns original thrown values.
// Native live-read scope: ordinary arrays with scalar own data entries. Check
// retained UTF-16 size before joining; repeated references to one large string
// must not create an unchecked product allocation. Exotic joins/accessors are
// explicit unsupported cases, never invoked as part of this preflight.
const intrinsicArrayJoin = Array.prototype.join;
const IntrinsicDate = Date;
const intrinsicDefineProperty = Object.defineProperty;
// The allocator owns no values. Tokens are unique across owners and never reused.
let nextRetainedToken = 1;
const retainedValues = new WeakMap();
export function releaseLiveRenderValues(bridge) {
  const values = retainedValues.get(bridge);
  if (values) values.clear();
  retainedValues.delete(bridge);
}
function boundedOrdinaryJoin(value, separator) {
  const unsupported = () => { throw new TypeError('Live array join requires ordinary scalar data entries'); };
  if (!Array.isArray(value) || Object.getPrototypeOf(value) !== Array.prototype ||
      Object.getOwnPropertyDescriptor(value, 'join') !== undefined ||
      Object.getOwnPropertyDescriptor(Array.prototype, 'join')?.value !== intrinsicArrayJoin ||
      typeof separator !== 'string') unsupported();
  const length = Object.getOwnPropertyDescriptor(value, 'length')?.value;
  if (!Number.isSafeInteger(length) || length < 0) unsupported();
  if (length > 1_000_000) return null;
  const parts = [];
  let units = 0;
  for (let index = 0; index < length; index++) {
    const descriptor = Object.getOwnPropertyDescriptor(value, index);
    if (descriptor !== undefined && !Object.hasOwn(descriptor, 'value')) unsupported();
    // A hole is empty only without inherited indexed data/accessors.
    if (descriptor === undefined && index in value) unsupported();
    const item = descriptor?.value;
    let text;
    if (item === undefined || item === null) text = '';
    else if (typeof item === 'string') text = item;
    else if (typeof item === 'number' || typeof item === 'boolean') text = `${item}`;
    else unsupported();
    units += (index === 0 ? 0 : separator.length) + text.length;
    if (units > 16_000_000) return null;
    parts.push(text);
  }
  return Reflect.apply(intrinsicArrayJoin, parts, [separator]);
}
export function executeLiveRead(read, operation, value, key, context) {
  return read(operation, value, key, context);
}
export function liveRenderRead(capture, bridge) {
  const identities = new WeakMap();
  let nextIdentity = 1;
  const iterators = new WeakMap();
  let iteratorCount = 0, advances = 0;
  const object = value => value !== null && (typeof value === 'object' || typeof value === 'function');
  const close = record => {
    if (record.done) return;
    record.done = true;
    const method = record.iterator.return;
    if (method === undefined || method === null) return;
    const result = Reflect.apply(method, record.iterator, []);
    if (!object(result)) throw new TypeError('Iterator return result is not an object');
  };
  return (operation, value, key, context) => {
    try {
      switch (operation) {
        case 0: case 1: return value[key];
        case 2: return `${value}`;
        case 5: return key in value;
        case 6: return {...value};
        case 7: return value.get(key);
        case 9: return boundedOrdinaryJoin(value, key);
        case 13: {
          const [receiver, args, notCallable, dateIndices = [], records = []] = key;
          // Native code supplies scalar milliseconds. Each argument becomes one
          // fresh Date before the call, so retained host references keep identity.
          for (const index of dateIndices) args[index] = new IntrinsicDate(args[index]);
          for (const [index, fields] of records) {
            const record = {};
            for (const [name, field] of fields) intrinsicDefineProperty(record, name,
              {value: field, writable: true, enumerable: true, configurable: true});
            args[index] = record;
          }
          if (typeof value !== 'function') throw new TypeError(notCallable);
          return Reflect.apply(value, receiver, args);
        }
        case 14: {
          if (iteratorCount === 1_000_000) return null;
          const method = value[Symbol.iterator];
          const iterator = Reflect.apply(method, value, []);
          if (!object(iterator)) throw new TypeError('Iterator result is not an object');
          const next = iterator.next;
          const token = Object.create(null);
          iterators.set(token, {iterator, next, done: false});
          iteratorCount++;
          return token;
        }
        case 15: {
          const record = iterators.get(value);
          if (record === undefined) throw new TypeError('Foreign live iterator');
          if (record.done) return [true, undefined];
          if (advances === 8_000_000) return null;
          advances++;
          let result;
          try {
            result = Reflect.apply(record.next, record.iterator, []);
            if (!object(result)) throw new TypeError('Iterator next result is not an object');
            if (result.done) { record.done = true; return [true, undefined]; }
            return [false, result.value];
          } catch (error) { record.done = true; throw error; }
        }
        case 16: {
          const record = iterators.get(value);
          if (record === undefined) throw new TypeError('Foreign live iterator');
          close(record);
          return undefined;
        }
        case 17: {
          if (!Number.isInteger(key) || key < 0 || key > 0xffffffff ||
              !bridge || !Object.hasOwn(bridge.objects, key)) {
            throw new TypeError('Invalid retained graph node');
          }
          return bridge.objects[key];
        }
        case 19: {
          if (!bridge || bridge.disposed) throw new TypeError('Retained live value owner is disposed');
          let values = retainedValues.get(bridge);
          if ((values?.size ?? 0) >= 1_000_000 || nextRetainedToken > 0xffffffff) return null;
          if (!values) { values = new Map(); retainedValues.set(bridge, values); }
          const token = nextRetainedToken++;
          values.set(token, value);
          return token;
        }
        case 20: {
          const values = bridge && !bridge.disposed && retainedValues.get(bridge);
          if (!Number.isInteger(key) || key < 1 || key > 0xffffffff || !values || !values.has(key))
            throw new TypeError('Foreign or disposed retained live value');
          return values.get(key);
        }
        case 18: return +value;
        case 12: return Array.isArray(value);
        case 10: {
          const keys = Object.keys(value);
          if (keys.length > 1_000_000) return null;
          let units = 0;
          for (const name of keys) {
            units += name.length;
            if (units > 16_000_000) return null;
          }
          return keys;
        }
        case 11: {
          const captured = [];
          for (const item of value.values()) {
            if (captured.length === key) return null;
            captured.push(item);
          }
          return captured;
        }
        case 8: {
          if ((typeof value !== 'object' || value === null) && typeof value !== 'function') return undefined;
          const existing = identities.get(value);
          if (existing !== undefined) return existing;
          if (nextIdentity > 1_000_000) return null; // Native budget error, not host throw.
          const identity = nextIdentity++;
          identities.set(value, identity);
          return identity;
        }
        case 4: {
          const captured=[];
          for(const item of value){
            if(captured.length===key)return null; // Native budget error, not host throw.
            captured.push(item);
          }
          return captured;
        }
        case 3: {
          if (typeof value === 'string') return value;
          // Run the pinned first string operation in this engine. In particular,
          // do not stringify a missing or numeric label and hide its TypeError.
          const receiver = value;
          switch (key) {
            case 'trim': return receiver.trim();
            case 'split': return receiver.split(" ");
            case 'toLowerCase': return receiver.toLowerCase();
            case 'indexOf': return receiver.indexOf(":");
            case 'replace': return receiver.replace("", "");
            case 'includes': break;
            default: throw new TypeError('Unknown live string operation');
          }
          if (context === 1) { const t = value; t.includes("&"); }
          else { const e = value, vx = "¶ß"; e.includes(vx); }
          return value;
        }
        default: throw new TypeError('Unknown live render operation');
      }
    } catch (error) {
      // IteratorClose runs with an existing abrupt completion. Its secondary
      // failure must not replace a native original error in the JS throw lane.
      if (operation !== 16) capture(error);
      throw error;
    }
  };
}

// Readonly completion: admission counts the old complete graph plus metadata,
// but finishing a render does not authorize replacing caller-owned objects.
// response[2] is the canonical frame size with a zero metadata length; the
// inner success packet carries the separately counted legacy metadata bytes.
export function detachedRenderResult(response, revision, raw, type) {
  const invalid = () => { throw new TypeError('Invalid detached render packet'); };
  if (!Array.isArray(response) || response.length !== 5 || response[0] !== 1 ||
      response[1] !== revision || typeof revision !== 'string' || !/^(0|[1-9][0-9]*)$/.test(revision) ||
      !Number.isSafeInteger(response[2]) || response[2] < 0 || response[2] > 64 * 1024 * 1024 ||
      (response[3] !== 0 && response[3] !== 1)) invalid();
  if (response[3] === 0) {
    if (typeof response[4] !== 'string') invalid();
    const outcome = JSON.parse(response[4]);
    if (!outcome || typeof outcome !== 'object' || !Object.hasOwn(outcome, 'Err') || Object.keys(outcome).length !== 1) invalid();
    return outcome;
  }
  const packet = response[4];
  if (!Array.isArray(packet) || packet.length !== 3 || packet[0] !== 1 ||
      !Number.isSafeInteger(packet[1]) || packet[1] < 0 || response[2] + packet[1] > 64 * 1024 * 1024) invalid();
  const value = packet[2], keys = raw ? ['svg', 'width', 'height', 'links'] : ['type', 'svg', 'width', 'height', 'links'];
  const actual = value && Object.keys(value);
  if (!value || Object.getPrototypeOf(value) !== Object.prototype || actual.length !== keys.length ||
      !keys.every((key, index) => actual[index] === key) || typeof value.svg !== 'string' ||
      typeof value.width !== 'number' || typeof value.height !== 'number' || !Array.isArray(value.links) ||
      (!raw && value.type !== type)) invalid();
  for (const link of value.links) {
    const actual = link && Object.keys(link);
    if (!link || Object.getPrototypeOf(link) !== Object.prototype || actual.length !== 3 ||
        actual[0] !== 'id' || actual[1] !== 'href' || actual[2] !== 'internal' ||
        typeof link.id !== 'string' || typeof link.href !== 'string' || typeof link.internal !== 'boolean') invalid();
  }
  return {Ok: value};
}
