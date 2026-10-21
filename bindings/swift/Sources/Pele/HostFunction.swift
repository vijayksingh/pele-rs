import Foundation
import CPele

extension Pele {
    /// An ordinary owner-thread callable. Replacing call changes future invocations
    /// of this same function identity. Bound documents retain it until close.
    public final class HostFunction {
        public var call: (ActiveModel, PublicGraph.Atom, [PublicGraph.Atom]) throws -> PublicGraph.Atom
        public init(_ call: @escaping (ActiveModel, PublicGraph.Atom, [PublicGraph.Atom]) throws -> PublicGraph.Atom) { self.call = call }
    }
    /// Borrowed access to the authoritative arena during a host method. Keeping
    /// this object is safe, but every access after callback return throws.
    public final class ActiveModel {
        private var handle: OpaquePointer?
        private var active = false
        private var takeResourceError: (() -> (any Error)?)?
        private var resolveHostError: ((UInt32) -> (any Error)?)?
        fileprivate init(_ handle: OpaquePointer, takeResourceError: @escaping () -> (any Error)?, resolveHostError: @escaping (UInt32) -> (any Error)?) {
            self.handle = handle; self.takeResourceError = takeResourceError; self.resolveHostError = resolveHostError
        }
        fileprivate func finish() { handle = nil; takeResourceError = nil; resolveHostError = nil }
        private func access(_ request: AccessRequest) throws -> ActiveValue {
            guard let handle else { throw PeleError(code: "semantic", message: "Active model callback has finished") }
            guard !active else { throw PeleError(code: "semantic", message: "Active model view is already in use") }
            active = true; defer { active = false }
            let bytes = try JSONEncoder().encode(request)
            guard bytes.count <= 1024 * 1024 else { throw PeleError(code: "limit", message: "Active model request exceeds byte budget") }
            _ = takeResourceError?()
            do {
                let result = bytes.withUnsafeBytes { pele_active_model_access_v12(handle, $0.bindMemory(to: UInt8.self).baseAddress, $0.count) }
                let outcome = try Pele.compatibilityResult(result, as: AccessOutcome.self)
                switch outcome {
                case .value(let value): return value
                case .error(.hostException(let token)):
                    if let original = resolveHostError?(token) { throw original }
                    throw PublicOperationFailure(value: .hostException(token))
                case .error(.pele(let error)): throw error
                case .error(let error): throw PublicOperationFailure(value: error)
                }
            } catch { throw takeResourceError?() ?? error }
        }
        private func atom(_ request: AccessRequest) throws -> PublicGraph.Atom {
            guard case .atom(let value) = try access(request) else { throw PeleError(code: "internal", message: "Active operation did not return an Atom") }
            return value
        }
        public func root() throws -> PublicGraph.Atom { try atom(.init(operation: "root")) }
        public func read(_ key: String, on receiver: PublicGraph.Atom) throws -> PublicGraph.Atom {
            try atom(.init(operation: "read", receiver: receiver, key: key))
        }
        public func set(_ key: String, on receiver: PublicGraph.Atom, to value: PublicGraph.Atom) throws {
            _ = try atom(.init(operation: "set", receiver: receiver, key: key, value: value))
        }
        public func intern(_ value: JSString) throws -> PublicGraph.Atom { try atom(.init(operation: "intern", units: value)) }
        public func intern(_ value: String) throws -> PublicGraph.Atom { try intern(JSString(value)) }
        public func string(_ value: PublicGraph.Atom) throws -> JSString {
            guard case .string(let value) = try access(.init(operation: "string", value: value)) else { throw PeleError(code: "internal", message: "Active operation did not return a string") }
            return value
        }
        public func object() throws -> PublicGraph.Atom { try atom(.init(operation: "object")) }
        public func array() throws -> PublicGraph.Atom { try atom(.init(operation: "array")) }
        public func date(milliseconds: Double) throws -> PublicGraph.Atom { try atom(.init(operation: "date", value: .number(milliseconds))) }
        public func dateMilliseconds(_ value: PublicGraph.Atom) throws -> Double {
            guard case .number(let result) = try atom(.init(operation: "dateMillis", value: value)) else { throw PeleError(code: "internal", message: "Active Date operation did not return a number") }
            return result
        }
        public func append(_ value: PublicGraph.Atom, to array: PublicGraph.Atom) throws { _ = try atom(.init(operation: "push", receiver: array, value: value)) }
        /// Calls a captured native method with explicit receiver. It never
        /// resolves or recursively invokes a replaced own host function.
        public func invokeBuiltin(method: UInt32, receiver: PublicGraph.Atom, arguments: [PublicGraph.Atom] = []) throws -> PublicGraph.Atom {
            try atom(.init(operation: "invokeBuiltin", call: .init(method: method, receiver: receiver, arguments: arguments)))
        }
    }
}

extension Pele.MutableDocument {
    // Shared with raw/prepared rendering: retain original errors through result
    // publication, then clear them once the outer operation has consumed them.
    // A nested operation may consume/clear its own failure without erasing a
    // pending outer failure on the same bound callable.
    func withHostFunctionErrorScope<T>(_ body: () throws -> T) rethrows -> T {
        let saved = hostFunctions.values.map { ($0, $0.error) }
        let opaqueErrors = hostValueAccess?.errors
        defer {
            for (box, error) in saved { box.error = error }
            if let opaqueErrors { hostValueAccess?.errors = opaqueErrors }
        }
        return try body()
    }
    func withHostFunctionResources<T>(_ takeError: () -> (any Error)?, _ body: () -> T) -> T {
        withoutActuallyEscaping(takeError) { errors in
            let saved = hostFunctions.values.map { ($0, $0.takeResourceError, $0.resolveHostError) }
            for box in hostFunctions.values {
                box.error = nil; box.takeResourceError = errors
                box.resolveHostError = { [weak self] token in self?.hostFunctions[token]?.error ?? self?.hostValueAccess?.errors[token] }
            }
            defer {
                for (box, take, resolve) in saved {
                    box.takeResourceError = take; box.resolveHostError = resolve
                }
            }
            return body()
        }
    }
    func originalHostFunctionError(_ result: Pele.PublicRenderResult) -> (any Error)? {
        if case .thrown(.hostException(let token)) = result { return hostFunctions[token]?.error ?? hostValueAccess?.errors[token] }
        return nil
    }
    func clearHostFunctionErrors() { for box in hostFunctions.values { box.error = nil } }
    /// Binds callable identity; set the returned Atom on an ordinary graph field
    /// and apply that graph through the existing revision API.
    public func bindHostFunction(_ function: Pele.HostFunction) throws -> Pele.PublicGraph.Atom {
        let box = HostFunctionBox(function)
        var table = PeleHostFunctionV12()
        table.version = 12; table.struct_size = UInt32(MemoryLayout<PeleHostFunctionV12>.size)
        table.context = Unmanaged.passUnretained(box).toOpaque()
        table.retain = hostFunctionRetain; table.release = hostFunctionRelease; table.invoke = hostFunctionInvoke
        struct Bound: Decodable { let reference: UInt32 }
        return try withHandle { handle in
            var reference: UInt32 = 0
            let result = try withExtendedLifetime(box) { try Pele.compatibilityResult(pele_public_bind_host_function_v12(handle, &table, &reference), as: Bound.self) }
            guard reference != 0, result.reference == reference else { throw PeleError(code: "internal", message: "Invalid host function reference") }
            box.reference = reference; hostFunctions[reference] = box
            return .hostReference(reference)
        }
    }
}

private struct AccessRequest: Encodable {
    let operation: String
    var receiver: Pele.PublicGraph.Atom? = nil
    var key: String? = nil
    var value: Pele.PublicGraph.Atom? = nil
    var units: Pele.JSString? = nil
    var call: Call? = nil
    struct Call: Encodable { let method: UInt32; let receiver: Pele.PublicGraph.Atom; let arguments: [Pele.PublicGraph.Atom] }
}
private enum ActiveValue: Decodable {
    case atom(Pele.PublicGraph.Atom), string(Pele.JSString)
    private enum Keys: String, CodingKey { case kind, value }
    init(from decoder: any Decoder) throws {
        let c = try decoder.container(keyedBy: Keys.self)
        switch try c.decode(String.self, forKey: .kind) {
        case "atom": self = .atom(try c.decode(Pele.PublicGraph.Atom.self, forKey: .value))
        case "string": self = .string(try c.decode(Pele.JSString.self, forKey: .value))
        default: throw PeleError(code: "internal", message: "Invalid active model value")
        }
    }
}
private enum AccessOutcome: Decodable {
    case value(ActiveValue), error(Pele.PublicOperationError)
    private enum Keys: String, CodingKey { case Ok, Err }
    init(from decoder: any Decoder) throws {
        let c = try decoder.container(keyedBy: Keys.self)
        if c.contains(.Ok) { self = .value(try c.decode(ActiveValue.self, forKey: .Ok)) }
        else { self = .error(try c.decode(Pele.PublicOperationError.self, forKey: .Err)) }
    }
}
final class HostFunctionBox {
    let function: Pele.HostFunction
    var reference: UInt32 = 0
    var error: (any Error)?
    var takeResourceError: (() -> (any Error)?)?
    var resolveHostError: ((UInt32) -> (any Error)?)?
    init(_ function: Pele.HostFunction) { self.function = function }
}
private func hostFunctionRetain(_ p: UnsafeMutableRawPointer?) { if let p { _ = Unmanaged<HostFunctionBox>.fromOpaque(p).retain() } }
private func hostFunctionRelease(_ p: UnsafeMutableRawPointer?) { if let p { Unmanaged<HostFunctionBox>.fromOpaque(p).release() } }
private func hostFunctionBytesRelease(_ p: UnsafeMutableRawPointer?) { if let p { Unmanaged<NSData>.fromOpaque(p).release() } }
private func hostFunctionInvoke(_ context: UnsafeMutableRawPointer?, _ view: OpaquePointer?, _ bytes: UnsafePointer<UInt8>?, _ count: Int, _ reply: UnsafeMutablePointer<PeleHostFunctionReplyV12>?) {
    guard let context, let view, let reply else { return }
    let box = Unmanaged<HostFunctionBox>.fromOpaque(context).takeUnretainedValue()
    let model = Pele.ActiveModel(view, takeResourceError: { box.takeResourceError?() }, resolveHostError: { box.resolveHostError?($0) })
    defer { model.finish() }
    do {
        struct Call: Decodable { let receiver: Pele.PublicGraph.Atom; let arguments: [Pele.PublicGraph.Atom] }
        guard let bytes, count <= 1024 * 1024 else { throw PeleError(code: "internal", message: "Invalid host function arguments") }
        let call = try JSONDecoder().decode(Call.self, from: Data(bytes: bytes, count: count))
        let value = try box.function.call(model, call.receiver, call.arguments)
        let data = try JSONEncoder().encode(value) as NSData
        reply.pointee.payload = PeleForeignBytesV10(owner: Unmanaged.passRetained(data).toOpaque(), data: data.bytes.assumingMemoryBound(to: UInt8.self), len: data.length, release: hostFunctionBytesRelease)
    } catch { box.error = error; reply.pointee.status = 2; reply.pointee.exception = box.reference }
}
