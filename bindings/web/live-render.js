// Host values stay opaque across resource callbacks. Rust chooses every field
// and render phase; this adapter supplies ordinary JavaScript value operations.
// Typed operands avoid converting the same control strings on every property
// read. The callback itself remains job-local and owns original thrown values.
export function executeLiveRead(read, operation, value, key, context) {
  return read(operation, value, key, context);
}
export function liveRenderRead(capture) {
  return (operation, value, key, context) => {
    try {
      switch (operation) {
        case 0: case 1: return value[key];
        case 2: return `${value}`;
        case 5: return key in value;
        case 6: return {...value};
        case 7: return value.get(key);
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
          if (key !== 'includes') throw new TypeError('Unknown live string operation');
          if (context === 1) { const t = value; t.includes("&"); }
          else { const e = value, vx = "¶ß"; e.includes(vx); }
          return value;
        }
        default: throw new TypeError('Unknown live render operation');
      }
    } catch (error) { capture(error); throw error; }
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
