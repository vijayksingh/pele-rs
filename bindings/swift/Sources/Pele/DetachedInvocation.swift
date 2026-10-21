import Foundation
import CPele

private struct InvocationSelection: Decodable { let selected: Bool }
private struct InvocationStep: Decodable {
    let complete: Bool?
    let host: HostOperation?
}
private struct HostOperation: Decodable {
    let kind: String
    let receiver: Pele.PublicGraph.Atom?
    let key: String?
    let function: Pele.PublicGraph.Atom?
    let arguments: [Pele.PublicGraph.Atom]?
    let value: Pele.PublicGraph.Atom?
    let left: Pele.PublicGraph.Atom?
    let right: Pele.PublicGraph.Atom?
    let notCallableError: String?
    let result: String?
}
private struct HostTypeError: Error { let message: String }
private func invalidHost(_ message: String) -> PeleError { PeleError(code: "semantic", message: message) }

extension Pele.MutableDocument {
    func invokeDetached(_ call: Data) throws -> Pele.PublicInvocationOutcome? {
        var job: OpaquePointer?
        let selected: InvocationSelection
        do {
            selected = try withHandle { handle in
                try call.withUnsafeBytes { bytes in
                    try Pele.compatibilityResult(pele_public_invocation_prepare_v17(handle, revision,
                        bytes.bindMemory(to: UInt8.self).baseAddress, bytes.count, &job), as: InvocationSelection.self)
                }
            }
        } catch { if let job { pele_public_invocation_free_v17(job) }; throw error }
        guard selected.selected else { if let job { pele_public_invocation_free_v17(job) }; return nil }
        guard let job else { throw invalidHost("Missing public invocation job") }
        defer { pele_public_invocation_free_v17(job) }
        return try withDetachedLifetime { _ in
            var reply: Pele.PublicHostValue = .undefined
            var replyState: UInt32 = 0
            var original: (any Error)?
            var temporary: [UInt32] = []
            var retainedReply: UInt32?
            do {
                while true {
                    let exchange = try withHandle { handle in
                        var acknowledged = revision
                        let response: OpaquePointer?
                        if replyState == 2 || retainedReply != nil {
                            var packet = PeleHostValueV17()
                            packet.version = 17; packet.struct_size = UInt32(MemoryLayout<PeleHostValueV17>.size)
                            packet.tag = 5; packet.payload = UInt64(retainedReply ?? 0)
                            response = pele_public_invocation_resume_v17(handle, job, replyState, &packet, &acknowledged)
                        } else {
                            response = try reply.withCValue(store: hostValueStore, temporary: &temporary) { packet in
                                pele_public_invocation_resume_v17(handle, job, replyState, packet, &acknowledged)
                            }
                        }
                        acceptPreparedRevision(acknowledged)
                        return try Pele.binaryResult(response)
                    }
                    retainedReply = nil
                    acceptPreparedRevision(exchange.revision)
                    let step = try JSONDecoder().decode(InvocationStep.self, from: exchange.metadata)
                    if step.complete == true { break }
                    guard let operation = step.host else { throw invalidHost("Missing native host operation") }
                    do {
                        let returned = try perform(operation, graph: exchange.graph)
                        replyState = 1
                        if operation.kind == "call", operation.result == "discard" { reply = .undefined }
                        else if operation.kind == "call", operation.result == "retain" {
                            let id = try hostValueStore.retain(returned, reuseIdentity: false)
                            temporary.append(id); retainedReply = id; reply = .undefined
                        } else { reply = returned }
                    } catch let failure as HostTypeError {
                        replyState = 3; reply = .string(.init(failure.message))
                    } catch {
                        original = error; replyState = 2; reply = .undefined
                    }
                }
                let exchange = try withHandle { handle in
                    var acknowledged = revision
                    let response = pele_public_invocation_finish_v17(handle, job, &acknowledged)
                    acceptPreparedRevision(acknowledged)
                    return try Pele.binaryResult(response)
                }
                acceptPreparedRevision(exchange.revision)
                let outcome = Pele.PublicInvocationOutcome(revision: exchange.revision, graph: exchange.graph,
                    result: try JSONDecoder().decode(Pele.PublicOperationResult.self, from: exchange.metadata))
                let returned: Pele.PublicGraph.Atom?
                if case .returned(let atom) = outcome.result { returned = atom } else { returned = nil }
                hostValueStore.release(temporary, preserving: exchange.graph, result: returned)
                if let original { throw original }
                return outcome
            } catch { throw original ?? error }
        }
    }
    private func perform(_ operation: HostOperation, graph: Pele.PublicGraph) throws -> Pele.PublicHostValue {
        func value(_ atom: Pele.PublicGraph.Atom?) throws -> Pele.PublicHostValue {
            guard let atom else { throw invalidHost("Missing host operation value") }
            return try hostValue(atom, graph: graph)
        }
        switch operation.kind {
        case "get":
            guard let key = operation.key else { throw invalidHost("Missing host field key") }
            return try getHost(value(operation.receiver), key: key)
        case "call":
            let function = try value(operation.function), receiver = try value(operation.receiver)
            let arguments = try (operation.arguments ?? []).map { try hostValue($0, graph: graph) }
            if case .host(let object) = function, let callable = object as? any Pele.PublicHostCallable {
                return try callable.publicCall(receiver: receiver, arguments: arguments)
            }
            if case .builtinFunction(let method) = function {
                let snapshot = try snapshot(); var graph = snapshot.graph
                func atom(_ value: Pele.PublicHostValue) throws -> Pele.PublicGraph.Atom {
                    switch value {
                    case .undefined: return .undefined
                    case .null: return .null
                    case .bool(let b): return .bool(b)
                    case .number(let n): return .number(n)
                    case .string(let s): return .string(try graph.intern(s))
                    case .graphReference(let id): return .ref(id)
                    case .builtinFunction(let id): return .builtinFunction(id)
                    case .host: return .hostReference(try hostValueStore.retain(value))
                    }
                }
                let receiver = try atom(receiver), arguments = try arguments.map(atom)
                try apply(graph, expectedRevision: snapshot.revision)
                let outcome = try invoke(method: method, receiver: receiver, arguments: arguments)
                switch outcome.result {
                case .returned(let atom): return try hostValue(atom, graph: outcome.graph)
                case .thrown(let error): throw Pele.PublicOperationFailure(value: error)
                }
            }
            throw HostTypeError(message: operation.notCallableError ?? "Host value is not callable")
        case "truthy": return .bool(try truthy(value(operation.value)))
        case "toString": return .string(try stringHost(value(operation.value)))
        case "greaterEqual":
            let left = try primitiveHost(value(operation.left), hint: .number)
            let right = try primitiveHost(value(operation.right), hint: .number)
            if case .string(let a) = left, case .string(let b) = right {
                return .bool(!a.codeUnits.lexicographicallyPrecedes(b.codeUnits))
            }
            return .bool(try numberHost(left) >= numberHost(right))
        case "strictEqual": return .bool(try equalHost(value(operation.left), value(operation.right)))
        default: throw invalidHost("Unsupported native host operation")
        }
    }
    private func getHost(_ value: Pele.PublicHostValue, key: String) throws -> Pele.PublicHostValue {
        switch value {
        case .host(let object): return try object.publicGet(.init(key))
        case .graphReference(let id):
            let snapshot = try snapshot()
            return try hostValue(snapshot.graph.field(key, on: id) ?? .undefined, graph: snapshot.graph)
        case .string(let text):
            if key == "length" { return .number(Double(text.codeUnits.count)) }
            if let i = Int(key), String(i) == key, text.codeUnits.indices.contains(i) { return .string(.init(codeUnits: [text.codeUnits[i]])) }
            return .undefined
        case .undefined: throw HostTypeError(message: "Cannot read properties of undefined (reading '\(key)')")
        case .null: throw HostTypeError(message: "Cannot read properties of null (reading '\(key)')")
        default: return .undefined
        }
    }
    private func primitiveHost(_ value: Pele.PublicHostValue, hint: Pele.PublicHostHint) throws -> Pele.PublicHostValue {
        if case .host(let object) = value {
            let result = try object.publicPrimitive(hint)
            switch result { case .host, .graphReference, .builtinFunction: throw HostTypeError(message: "Cannot convert object to primitive value"); default: return result }
        }
        if case .graphReference = value { throw invalidHost("Graph-coordinate object coercion requires an explicit PublicHostObject") }
        if case .builtinFunction = value { throw invalidHost("Builtin function string conversion is unavailable") }
        return value
    }
    private func truthy(_ value: Pele.PublicHostValue) -> Bool {
        switch value {
        case .undefined, .null: return false
        case .bool(let b): return b
        case .number(let n): return n != 0 && !n.isNaN
        case .string(let s): return !s.codeUnits.isEmpty
        case .host(let object): return object.publicTruthy
        default: return true
        }
    }
    private func stringHost(_ value: Pele.PublicHostValue) throws -> Pele.JSString {
        switch try primitiveHost(value, hint: .string) {
        case .undefined: return .init("undefined")
        case .null: return .init("null")
        case .bool(let b): return .init(b ? "true" : "false")
        case .string(let s): return s
        case .number(let n): return try Pele.compatibilityResult(pele_host_number_string_v17(n.bitPattern), as: Pele.JSString.self)
        default: throw invalidHost("Cannot convert host value to string")
        }
    }
    private func numberHost(_ value: Pele.PublicHostValue) throws -> Double {
        var temporary: [UInt32] = []
        let atom: Pele.PublicGraph.Atom = try value.withCValue(store: hostValueStore, temporary: &temporary) { packet in
            try Pele.compatibilityResult(pele_host_primitive_number_v17(packet), as: Pele.PublicGraph.Atom.self)
        }
        guard case .number(let number) = atom else { throw invalidHost("Invalid numeric host reply") }
        return number
    }
    private func equalHost(_ left: Pele.PublicHostValue, _ right: Pele.PublicHostValue) -> Bool {
        switch (left, right) {
        case (.undefined,.undefined),(.null,.null): return true
        case (.bool(let a),.bool(let b)): return a == b
        case (.number(let a),.number(let b)): return a == b
        case (.string(let a),.string(let b)): return a == b
        case (.host(let a),.host(let b)): return a === b
        case (.graphReference(let a),.graphReference(let b)),(.builtinFunction(let a),.builtinFunction(let b)): return a == b
        default: return false
        }
    }
}
