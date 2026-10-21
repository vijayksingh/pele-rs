import Foundation
import CPele

private struct InvocationSelection: Decodable { let selected: Bool }
struct InvocationStep: Decodable {
    let complete: Bool?
    let host: HostOperation?
}
struct HostOperation: Decodable {
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
struct HostTypeError: Error { let message: String }
func invalidHost(_ message: String) -> PeleError { PeleError(code: "semantic", message: message) }

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
}
