import Foundation
import CPele

private struct ConstructionResult: Decodable {
    let error: Pele.PublicOperationError?
    private enum Keys: String, CodingKey { case Ok, Err }
    init(from decoder: any Decoder) throws {
        let values = try decoder.container(keyedBy: Keys.self)
        if values.contains(.Err) { error = try values.decode(Pele.PublicOperationError.self, forKey: .Err) }
        else if values.contains(.Ok) { error = nil }
        else { throw invalidHost("Invalid construction outcome") }
    }
}
extension Pele {
    /// Unpublished native receiver. Confine to its creating owner/actor.
    /// Callbacks may inspect/edit/invoke this owner between native phases. A
    /// graph reference is a coordinate, not an implicit foreign object lease.
    public final class PublicConstruction {
        private var handle: OpaquePointer?
        private var active = false
        private var running = false
        var hostValueStore: PublicHostStore
        public let type: String
        public private(set) var revision: UInt64
        /// Exposed only after native finish. Non-nil can accompany a thrown body
        /// or export error; it does not mean constructor evaluation succeeded.
        public private(set) var retainedReceiver: MutableDocument?
        internal init(handle: OpaquePointer, type: String, revision: UInt64, store: PublicHostStore) {
            self.handle = handle; self.type = type; self.revision = revision; hostValueStore = store
        }
        deinit { if let handle { pele_public_construction_free_v18(handle) } }
        public func close() throws {
            guard !active && !running else { throw invalidHost("Constructor job is already in use") }
            if let handle { self.handle = nil; pele_public_construction_free_v18(handle) }
            hostValueStore = PublicHostStore()
            retainedReceiver = nil
        }
        private func withHandle<T>(_ body: (OpaquePointer) throws -> T) throws -> T {
            guard let handle else { throw invalidHost("Constructor job is closed") }
            guard !active else { throw invalidHost("Constructor job is already in use") }
            active = true; defer { active = false }
            return try withExtendedLifetime(self) { try body(handle) }
        }
        public func snapshot() throws -> PublicSnapshot {
            try withHandle { handle in
                let exchange = try Pele.binaryResult(pele_public_construction_snapshot_v18(handle))
                revision = exchange.revision
                return PublicSnapshot(type: type, revision: revision, graph: exchange.graph)
            }
        }
        @discardableResult public func apply(_ graph: PublicGraph, expectedRevision: UInt64) throws -> UInt64 {
            let bytes = try graph.binaryBytes(revision: expectedRevision)
            guard bytes.count <= 64 * 1024 * 1024 else { throw invalidHost("Public binary graph exceeds transport budget") }
            struct Applied: Decodable { let revision: String }
            return try withHandle { handle in
                let applied = try bytes.withUnsafeBytes { bytes in try Pele.compatibilityResult(pele_public_construction_apply_v18(handle, bytes.bindMemory(to: UInt8.self).baseAddress, bytes.count), as: Applied.self) }
                guard let next = UInt64(applied.revision) else { throw invalidHost("Invalid construction revision") }
                revision = next; return next
            }
        }
        public func retainHostValue(_ value: PublicHostValue) throws -> PublicGraph.Atom {
            try withHandle { _ in .hostReference(try hostValueStore.retain(value)) }
        }
        public func resolveHostValue(_ atom: PublicGraph.Atom) throws -> PublicHostValue {
            try hostValue(atom, graph: snapshot().graph)
        }
        /// Calls a native builtin on the provisional receiver. No foreign callback
        /// dispatch occurs under this call; native method errors preserve state.
        public func invoke(method: UInt32, receiver: PublicGraph.Atom, arguments: [PublicGraph.Atom] = []) throws -> PublicInvocationOutcome {
            struct Call: Encodable { let method: UInt32; let receiver: PublicGraph.Atom; let arguments: [PublicGraph.Atom] }
            let bytes = try JSONEncoder().encode(Call(method: method, receiver: receiver, arguments: arguments))
            return try withHandle { handle in
                var acknowledged = revision
                let response = bytes.withUnsafeBytes { bytes in pele_public_construction_invoke_v18(handle, revision, bytes.bindMemory(to: UInt8.self).baseAddress, bytes.count, &acknowledged) }
                revision = acknowledged
                let exchange = try Pele.binaryResult(response); revision = exchange.revision
                return PublicInvocationOutcome(revision: revision, graph: exchange.graph, result: try JSONDecoder().decode(PublicOperationResult.self, from: exchange.metadata))
            }
        }
        /// Evaluate ordered native-requested phases. No owner lock spans a host
        /// getter/call/coercion. Original callback errors are rethrown by identity.
        public func run() throws -> MutableDocument {
            guard !active && !running else { throw invalidHost("Constructor job is already in use") }
            guard handle != nil else { throw invalidHost("Constructor job is closed") }
            running = true; defer { running = false }
            return try withExtendedLifetime(self) {
                var reply = PublicHostValue.undefined, replyState: UInt32 = 0
                var retainedReply: UInt32?, temporary: [UInt32] = []
                var original: (any Error)?
                do {
                    while true {
                        let exchange = try withHandle { handle in
                            var acknowledged = revision
                            let response: OpaquePointer?
                            if replyState == 2 || retainedReply != nil {
                                var packet = PeleHostValueV17(); packet.version = 17; packet.struct_size = UInt32(MemoryLayout<PeleHostValueV17>.size)
                                packet.tag = 5; packet.payload = UInt64(retainedReply ?? 0)
                                response = pele_public_construction_resume_v18(handle, replyState, &packet, &acknowledged)
                            } else {
                                response = try reply.withCValue(store: hostValueStore, temporary: &temporary) { pele_public_construction_resume_v18(handle, replyState, $0, &acknowledged) }
                            }
                            revision = acknowledged
                            return try Pele.binaryResult(response)
                        }
                        retainedReply = nil; revision = exchange.revision
                        let step = try JSONDecoder().decode(InvocationStep.self, from: exchange.metadata)
                        if step.complete == true { break }
                        guard let operation = step.host else { throw invalidHost("Missing construction host operation") }
                        do {
                            let result = try perform(operation, graph: exchange.graph); replyState = 1
                            if operation.kind == "call", operation.result == "discard" { reply = .undefined }
                            else if operation.kind == "call", operation.result == "retain" {
                                let id = try hostValueStore.retain(result, reuseIdentity: false)
                                temporary.append(id); retainedReply = id; reply = .undefined
                            } else { reply = result }
                        } catch let error as HostTypeError { replyState = 3; reply = .string(.init(error.message)) }
                        catch { original = error; replyState = 2; reply = .undefined }
                    }
                    let exchange = try withHandle { handle in
                        var owner: OpaquePointer?, acknowledged = revision
                        let response = pele_public_construction_finish_v18(handle, &owner, &acknowledged)
                        revision = acknowledged
                        if let owner {
                            let document = MutableDocument(handle: owner, type: type, revision: revision, isConstructorCreated: true)
                            document.hostValueStore = hostValueStore; document.usesHostValues = true
                            retainedReceiver = document
                        }
                        return try Pele.binaryResult(response)
                    }
                    // Finish has transferred the receiver. The consumed job is
                    // freed after its call returns; a callback can never see a
                    // half-freed receiver.
                    if let handle { self.handle = nil; pele_public_construction_free_v18(handle) }
                    let result = try JSONDecoder().decode(ConstructionResult.self, from: exchange.metadata)
                    hostValueStore.release(temporary, preserving: exchange.graph)
                    if let original { throw original }
                    if let error = result.error { throw PublicOperationFailure(value: error) }
                    guard let receiver = retainedReceiver else { throw invalidHost("Missing constructed receiver") }
                    return receiver
                } catch { throw original ?? error }
            }
        }
    }
}
extension Pele.PublicConstruction: HostOperationOwner {
    func invokeHostBuiltin(method: UInt32, receiver: Pele.PublicGraph.Atom, arguments: [Pele.PublicGraph.Atom]) throws -> Pele.PublicInvocationOutcome {
        try invoke(method: method, receiver: receiver, arguments: arguments)
    }
    func readHostGraph(_ id: UInt32, key: String) throws -> Pele.PublicHostValue {
        let bytes = Data(key.utf8)
        let atom: Pele.PublicGraph.Atom = try withHandle { handle in
            try bytes.withUnsafeBytes { bytes in try Pele.compatibilityResult(pele_public_construction_get_v18(handle, id, bytes.bindMemory(to: UInt8.self).baseAddress, bytes.count), as: Pele.PublicGraph.Atom.self) }
        }
        return try hostValue(atom, graph: snapshot().graph)
    }
}
