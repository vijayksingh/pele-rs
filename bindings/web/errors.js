// Pele errors.ts, pinned 458322a5 (MIT, kepano). Stack traces remain host-owned.
export class PeleError extends Error {
  code;
  type;
  line;
  column;
  snippet;
  constructor(message, code = 'syntax', info = {}) {
    super(message);
    this.name = 'PeleError';
    this.code = code;
    this.type = info.type ?? null;
    this.line = info.line ?? 0;
    this.column = info.column ?? 0;
    this.snippet = info.snippet ?? '';
  }
}
// Transport decoding is separate from the public constructor's string argument.
export function fromNativeError(error) {
  return new PeleError(error.message, error.code, error);
}

// Captured intrinsics operate only on a private failure witness. Native code
// already performed the method and retained its partial writes; no caller data
// or callback is replayed to obtain the host engine's exception spelling.
const failureApply = Reflect.apply;
const failureDefine = Object.defineProperty;
const failureCreate = Object.create;
const failureSetPrototype = Object.setPrototypeOf;
const failurePush = Array.prototype.push;
const failureArrayPrototype = Array.prototype;
const failureInteger = Number.isInteger;
export function primitiveJavaScriptFailure(operation, index) {
  const array = [];
  if (operation === 'readonlyArrayPushLength') {
    failureDefine(array, 'length', {writable: false});
  } else if (operation === 'readonlyArrayPushIndex') {
    if (!failureInteger(index) || index < 0 || index > 0xffffffff) throw new TypeError('Invalid JavaScript failure index');
    // An inherited nonwritable target lets the actual push intrinsic attempt
    // this index without allocating a dense array or mutating any model.
    const prototype = failureCreate(failureArrayPrototype);
    failureDefine(prototype, `${index}`, {value: undefined, writable: false});
    failureSetPrototype(array, prototype);
    failureDefine(array, 'length', {value: index});
  } else throw new TypeError('Unknown JavaScript failure operation');
  try { failureApply(failurePush, array, [undefined]); }
  catch (error) { return [error.name, error.message]; }
  throw new TypeError('JavaScript failure witness unexpectedly succeeded');
}
