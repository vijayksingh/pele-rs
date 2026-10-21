import Foundation
import CPele

extension Pele {
    /// Lossless values in an actor-confined host store. Graph references are
    /// coordinates in this document, not JavaScript object lifetime leases.
    public enum PublicHostValue {
        case undefined, null, bool(Bool), number(Double), string(JSString)
        case host(any PublicHostObject), graphReference(UInt32), builtinFunction(UInt32)
    }
    public enum PublicHostHint { case number, string }
    /// Explicit host semantics; no Swift reflection or JSON enumeration occurs.
    /// Implement getters/coercion deliberately. Defaults describe a plain object.
    public protocol PublicHostObject: AnyObject {
        func publicGet(_ key: JSString) throws -> PublicHostValue
        func publicPrimitive(_ hint: PublicHostHint) throws -> PublicHostValue
        var publicTruthy: Bool { get }
    }
    public protocol PublicHostCallable: PublicHostObject {
        func publicCall(receiver: PublicHostValue, arguments: [PublicHostValue]) throws -> PublicHostValue
    }
}
extension Pele.PublicHostObject {
    public func publicGet(_ key: Pele.JSString) throws -> Pele.PublicHostValue { .undefined }
    public func publicPrimitive(_ hint: Pele.PublicHostHint) throws -> Pele.PublicHostValue { .string(.init("[object Object]")) }
    public var publicTruthy: Bool { true }
}

final class PublicHostStore {
    private var values: [UInt32: Pele.PublicHostValue] = [:]
    private var identities: [ObjectIdentifier: UInt32] = [:]
    func retain(_ value: Pele.PublicHostValue, reuseIdentity: Bool = true) throws -> UInt32 {
        if reuseIdentity, case .host(let object) = value,
           let existing = identities[ObjectIdentifier(object)] { return existing }
        guard values.count < 1_000_000 else { throw PeleError(code: "limit", message: "Owner host value budget exceeded") }
        let id = try Pele.compatibilityResult(pele_host_identity_v17(), as: UInt32.self)
        values[id] = value
        if reuseIdentity, case .host(let object) = value { identities[ObjectIdentifier(object)] = id }
        return id
    }
    func contains(_ id: UInt32) -> Bool { values[id] != nil }
    func value(_ id: UInt32) throws -> Pele.PublicHostValue {
        guard let value = values[id] else { throw PeleError(code: "semantic", message: "Unknown owner host identity") }
        return value
    }
    func release(_ ids: [UInt32], preserving graph: Pele.PublicGraph, result: Pele.PublicGraph.Atom? = nil) {
        var retained = Set<UInt32>()
        func atom(_ value: Pele.PublicGraph.Atom) { if case .hostReference(let id) = value { retained.insert(id) } }
        atom(graph.root)
        if let result { atom(result) }
        for node in graph.nodes {
            node.properties.forEach { atom($0.value) }
            switch node {
            case .map(let entries, _): entries.forEach { atom($0.key); atom($0.value) }
            case .set(let values, _): values.forEach(atom)
            default: break
            }
        }
        for id in ids where !retained.contains(id) { values.removeValue(forKey: id) }
    }
}

extension Pele.PublicHostValue {
    func withCValue<T>(store: PublicHostStore, temporary: inout [UInt32], _ body: (UnsafePointer<PeleHostValueV17>) throws -> T) throws -> T {
        var packet = PeleHostValueV17()
        packet.version = 17; packet.struct_size = UInt32(MemoryLayout<PeleHostValueV17>.size)
        switch self {
        case .undefined: packet.tag = 0
        case .null: packet.tag = 1
        case .bool(let value): packet.tag = 2; packet.payload = value ? 1 : 0
        case .number(let value): packet.tag = 3; packet.payload = value.bitPattern
        case .string(let text):
            guard text.codeUnits.count <= 16_000_000 else { throw PeleError(code: "limit", message: "Public model graph exceeds work budget") }
            packet.tag = 4
            return try text.codeUnits.withUnsafeBufferPointer { units in
                packet.units = units.baseAddress; packet.unit_count = units.count
                return try withUnsafePointer(to: &packet, body)
            }
        default:
            let id = try store.retain(self, reuseIdentity: false); temporary.append(id)
            packet.tag = 5; packet.payload = UInt64(id)
        }
        return try withUnsafePointer(to: &packet, body)
    }
}

extension Pele.MutableDocument {
    /// Retain an opaque value without reading it; use the returned atom in graph
    /// edits or method arguments. Identity is local to this document.
    public func retainHostValue(_ value: Pele.PublicHostValue) throws -> Pele.PublicGraph.Atom {
        try withHandle { _ in
            let atom = Pele.PublicGraph.Atom.hostReference(try hostValueStore.retain(value))
            usesHostValues = true
            return atom
        }
    }
    public func resolveHostValue(_ atom: Pele.PublicGraph.Atom) throws -> Pele.PublicHostValue {
        let snapshot = try snapshot()
        return try hostValue(atom, graph: snapshot.graph)
    }
}
