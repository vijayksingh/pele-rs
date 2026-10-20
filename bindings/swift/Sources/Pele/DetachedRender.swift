import Foundation
import CPele

private struct DetachedSelection: Decodable { let selected: Bool }
private struct DetachedDispatch: Decodable { let completed: Bool }

extension Pele.MutableDocument {
    // Only a concrete builtin capability selects this path. Native Ref IDs are
    // current graph coordinates; arbitrary replacement is not a JS object lease.
    // Presentation/image entrypoints continue through their existing adapters.
    func renderDetached(settings: RenderNumberEncoding, resources: Pele.HostResources,
                        rawConfig: Data? = nil) throws -> Pele.PublicRenderOutcome? {
        var job: OpaquePointer?
        let selection: DetachedSelection
        do {
            selection = try withHandle { handle in
                let result = settings.withPacket { numbers in
                    settings.optionsJSON.withUnsafeBytes { options in
                        if let rawConfig {
                            return rawConfig.withUnsafeBytes { config in
                                pele_public_prepare_detached_raw_v15(handle, revision,
                                    config.bindMemory(to: UInt8.self).baseAddress, config.count,
                                    options.bindMemory(to: UInt8.self).baseAddress, options.count,
                                    numbers, &job)
                            }
                        }
                        return pele_public_prepare_detached_render_v15(handle, revision,
                            options.bindMemory(to: UInt8.self).baseAddress, options.count,
                            numbers, settings.outputKind, settings.outputBits, &job)
                    }
                }
                return try Pele.compatibilityResult(result, as: DetachedSelection.self)
            }
        } catch {
            if let job { pele_detached_render_job_free_v15(job) }
            throw error
        }
        guard selection.selected else {
            if let job { pele_detached_render_job_free_v15(job) }
            return nil
        }
        guard let job else { throw PeleError(code: "internal", message: "Missing detached render job") }
        defer { pele_detached_render_job_free_v15(job) }
        return try withDetachedLifetime { _ in
            var dispatchError: (any Error)?
            let response = resources.withCapturedNativeResources { host, takeError in
                withHostFunctionResources(takeError) {
                    do {
                        let dispatched = try Pele.compatibilityResult(pele_detached_render_run_v15(job, host), as: DetachedDispatch.self)
                        guard dispatched.completed else { throw PeleError(code: "internal", message: "Detached render did not complete") }
                    } catch { dispatchError = error }
                    // Finish after dispatched operation errors too. No document
                    // guard was retained during the synchronous run above.
                    do {
                        return try withHandle { handle in
                            var acknowledged = revision
                            let result = pele_public_finish_detached_render_v15(handle, job, &acknowledged)
                            acceptPreparedRevision(acknowledged)
                            return result
                        }
                    } catch { dispatchError = dispatchError ?? error; return nil }
                }
            }
            defer { clearHostFunctionErrors() }
            do {
                if let dispatchError {
                    pele_result_free(response.handle)
                    throw dispatchError
                }
                let exchange = try Pele.binaryResult(response.handle)
                let outcome = Pele.PublicRenderOutcome(revision: exchange.revision, graph: exchange.graph,
                    result: try JSONDecoder().decode(Pele.PublicRenderResult.self, from: exchange.metadata))
                acceptPreparedRevision(outcome.revision)
                if let original = originalHostFunctionError(outcome.result) { throw original }
                if let error = response.error { throw error }
                return outcome
            } catch { throw response.error ?? error }
        }
    }
}
