import Foundation
import Pele
import XCTest

private final class GanttCalendarError: Error {}
private final class GanttCalendarHost: Pele.PublicHostCallable {
  var get: (Pele.JSString) throws -> Pele.PublicHostValue = { _ in .undefined }
  var call: (Pele.PublicHostValue, [Pele.PublicHostValue]) throws -> Pele.PublicHostValue = {
    _, _ in .undefined
  }
  var primitive: (Pele.PublicHostHint) throws -> Pele.PublicHostValue = { _ in .undefined }
  func publicGet(_ key: Pele.JSString) throws -> Pele.PublicHostValue { try get(key) }
  func publicCall(receiver: Pele.PublicHostValue, arguments: [Pele.PublicHostValue]) throws
    -> Pele.PublicHostValue
  { try call(receiver, arguments) }
  func publicPrimitive(_ hint: Pele.PublicHostHint) throws -> Pele.PublicHostValue {
    try primitive(hint)
  }
}

@MainActor private final class GanttCalendarResourceState {
  let document: Pele.MutableDocument
  let failure = GanttCalendarError()
  var calls = 0
  var fail = true
  init(_ document: Pele.MutableDocument) { self.document = document }
  func invoke(_ method: UInt32, _ text: String) throws -> Pele.PublicGraph.Atom {
    var graph = try document.snapshot().graph
    let argument = try graph.intern(text)
    try document.apply(graph, expectedRevision: document.revision)
    var resources = Pele.HostResources()
    resources.timeZone = TimeZone(secondsFromGMT: 0)!
    let outcome = try document.invoke(
      method: method, receiver: graph.root, arguments: [.string(argument)], resources: resources)
    guard case .returned(let value) = outcome.result else {
      throw PeleError(code: "test", message: "Nested helper failed")
    }
    return value
  }

}
@MainActor final class GanttCalendarCallbackTests: XCTestCase {
  private func expected(_ value: Any) throws -> ConfigValue {
    try JSONDecoder().decode(
      ConfigValue.self,
      from: JSONSerialization.data(withJSONObject: value, options: [.fragmentsAllowed]))
  }
  private func canonical(_ graph: Pele.PublicGraph, result: Pele.PublicGraph.Atom) -> ConfigValue {
    var ids: [UInt32: UInt32] = [:]
    var nodes: [ConfigValue] = []
    func string(_ id: UInt32) -> ConfigValue {
      let value = graph.strings[Int(id)].stringIfWellFormed
      XCTAssertNotNil(value)
      return .string(value ?? "invalid UTF16")
    }
    func atom(_ v: Pele.PublicGraph.Atom) -> ConfigValue {
      switch v {
      case .undefined: return .array([.string("undefined")])
      case .null: return .array([.string("null")])
      case .bool(let b): return .array([.string("bool"), .bool(b)])
      case .number(let n):
        return .array([.string("number"), .string(String(format: "%016llx", n.bitPattern))])
      case .string(let id): return .array([.string("string"), string(id)])
      case .ref(let id):
        if let known = ids[id] { return .array([.string("ref"), .number(Double(known))]) }
        let mapped = UInt32(nodes.count)
        ids[id] = mapped
        nodes.append(.null)
        let node = graph.nodes[Int(id)]
        var row: [String: ConfigValue]
        switch node {
        case .instance(let prototype, _):
          XCTAssertEqual(prototype, 1)
          row = ["kind": .string("instance")]
        case .object: row = ["kind": .string("object")]
        case .array: row = ["kind": .string("array")]
        case .map(let entries, _):
          row = [
            "kind": .string("map"),
            "entries": .array(entries.map { .array([atom($0.key), atom($0.value)]) }),
          ]
        case .set(let values, _):
          row = ["kind": .string("set"), "values": .array(values.map(atom))]
        case .date(let milliseconds, _):
          row = [
            "kind": .string("date"),
            "milliseconds": .string(String(format: "%016llx", milliseconds.bitPattern)),
          ]
        default:
          XCTFail("Unexpected constructor node")
          row = [:]
        }
        var properties: [ConfigValue] = node.properties.map {
          .array([string($0.key), atom($0.value), .number(Double($0.flags))])
        }
        if case .array(let length, _) = node,
          !node.properties.contains(where: {
            graph.strings[Int($0.key)].stringIfWellFormed == "length"
          })
        {
          properties.append(.array([.string("length"), atom(.number(Double(length))), .number(2)]))
        }
        row["properties"] = .array(properties)
        nodes[Int(mapped)] = .object(row)
        return .array([.string("ref"), .number(Double(mapped))])
      case .builtinFunction(let id):
        XCTAssertEqual(id, 0)
        return .array([.string("function"), .number(2)])
      case .hostReference: return .array([.string("host"), .string("options")])
      }
    }
    let root = atom(graph.root)
    let returned = atom(result)
    return .object(["root": root, "nodes": .array(nodes), "result": returned])
  }

  private func invoke(
    _ document: Pele.MutableDocument, _ method: UInt32, _ args: [Pele.PublicGraph.Atom]
  ) throws -> Pele.PublicGraph.Atom {
    let root = try document.snapshot().graph.root
    let outcome = try document.invoke(method: method, receiver: root, arguments: args)
    switch outcome.result {
    case .returned(let value): return value
    case .thrown(let error):
      throw PeleError(code: "test", message: "Unexpected Gantt error: \(error)")
    }
  }
  private func strings(_ document: Pele.MutableDocument, _ values: [String]) throws -> [Pele
    .PublicGraph.Atom]
  {
    var graph = try document.snapshot().graph
    let atoms = try values.map { Pele.PublicGraph.Atom.string(try graph.intern($0)) }
    try document.apply(graph, expectedRevision: document.revision)
    return atoms
  }
  private func addTask(_ document: Pele.MutableDocument, _ title: String, _ data: String) throws {
    _ = try invoke(document, 15, strings(document, [title, data]))
  }
  private func ref(_ value: Pele.PublicGraph.Atom?) throws -> UInt32 {
    guard case .ref(let id) = value else {
      throw PeleError(code: "test", message: "Expected task reference")
    }
    return id
  }
  private func firstTask(_ graph: Pele.PublicGraph) throws -> UInt32 {
    try ref(graph.field("0", on: ref(graph.field("tasks"))))
  }
  private func setDate(_ graph: inout Pele.PublicGraph, _ milliseconds: Double, on task: UInt32)
    throws
  {
    let id = UInt32(graph.nodes.count)
    graph.nodes.append(.date(milliseconds: milliseconds, properties: []))
    try graph.setField("endTime", value: .ref(id), on: task)
  }
  private func aliases(
    _ graph: Pele.PublicGraph, _ result: Pele.PublicGraph.Atom, _ tasks: [UInt32]
  ) throws -> [Bool] {
    try tasks.map { task in
      guard case .ref(let lhs) = result, case .ref(let rhs) = try graph.field("endTime", on: task)
      else { return false }
      return lhs == rhs
    }
  }
  func testCalendarStartMatchesWarningPhasesRetainedDatesAndRecovery() throws {
    var path = URL(fileURLWithPath: #filePath)
    for _ in 0..<5 { path.deleteLastPathComponent() }
    let fixture = try XCTUnwrap(
      JSONSerialization.jsonObject(
        with: Data(
          contentsOf: path.appendingPathComponent("fixtures/public-gantt-calendar-start.json")))
        as? [String: Any])
    XCTAssertEqual(
      fixture["oracleSHA256"] as? String,
      "d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d")
    let rows = try XCTUnwrap(fixture["cases"] as? [[String: Any]])
    XCTAssertEqual(rows.count, 17)
    for row in rows {
      let id = try XCTUnwrap(row["id"] as? String)
      let raw = try XCTUnwrap(row["raw"] as? String)
      let registry = try Pele.Registry.allBuiltins()
      let options = GanttCalendarHost()
      let warning = GanttCalendarHost()
      let factory = try XCTUnwrap(registry.captureConstructor(for: "gantt", prototype: 1))
      let document = try factory.construct(firstArgument: .host(options))
      factory.close()
      try registry.close()
      defer { try? document.close() }
      var graph = try document.snapshot().graph
      let format = try graph.intern(id == "timestamp" ? "X" : "YYYY-MM-DD")
      try graph.setField("dateFormat", value: .string(format))
      try graph.setField("now", value: .number(1_704_813_630_000))
      try document.apply(graph, expectedRevision: document.revision)
      try addTask(document, "Alpha", ":a,2024-01-01,1d")
      try addTask(document, "Beta", ":b,2024-01-02,1d")
      graph = try document.snapshot().graph
      let tasks = try ref(graph.field("tasks"))
      let a = try ref(graph.field("0", on: tasks))
      let b = try ref(graph.field("1", on: tasks))
      try setDate(&graph, 1_704_153_600_000, on: a)
      try setDate(&graph, 1_704_240_000_000, on: b)
      if id == "undefined-end" { try graph.setField("endTime", value: .undefined, on: a) }
      if id == "shadowed-known" || id == "shadowed-warn" {
        try graph.setField("warn", value: .null)
      }
      try document.apply(graph, expectedRevision: document.revision)
      let rawAtom = try XCTUnwrap(strings(document, [raw]).first)
      let original = GanttCalendarError()
      var trace: [[Any]] = []
      var reads = 0
      options.get = { key in
        XCTAssertEqual(key, .init("warn"))
        reads += 1
        trace.append(["get", reads])
        if id == "getter-edit" && reads == 1 {
          var edited = try document.snapshot().graph
          try self.setDate(&edited, 1_709_251_200_000, on: a)
          try document.apply(edited, expectedRevision: document.revision)
        }
        if (id == "first-get-throw" && reads == 1) || (id == "second-get-throw" && reads == 2) {
          throw original
        }
        if id == "warn-false" { return .bool(false) }
        if id == "second-not-callable" && reads == 2 { return .number(0) }
        return .host(warning)
      }
      warning.call = { receiver, arguments in
        guard case .host(let owner) = receiver, case .string(let message) = arguments.first else {
          throw PeleError(code: "test", message: "Wrong warning receiver/message")
        }
        XCTAssertTrue(owner === options)
        trace.append(["call", owner === options, message.codeUnits.map(Int.init)])
        XCTAssertThrowsError(try document.close())
        if id == "warn-reentry" {
          _ = try self.invoke(document, 14, self.strings(document, ["Nested"]))
          var edited = try document.snapshot().graph
          try self.setDate(&edited, 1_706_745_600_000, on: a)
          let map = try self.ref(edited.field("byId"))
          let props = edited.nodes[Int(map)].properties
          edited.nodes[Int(map)] = .map(entries: [], properties: props)
          try document.apply(edited, expectedRevision: document.revision)
          try self.addTask(document, "Replacement", ":a,2024-01-01,1d")
        }
        if id == "warn-throw" { throw original }
        return .undefined
      }
      var resources = Pele.HostResources()
      resources.timeZone = try XCTUnwrap(TimeZone(secondsFromGMT: 0))
      for phase in ["first", "recovery"] {
        if phase == "recovery" {
          options.get = { _ in .undefined }
          trace.removeAll()
          if id == "shadowed-known" || id == "shadowed-warn" {
            var edited = try document.snapshot().graph
            let root = try ref(edited.root)
            guard case .instance(let prototype, let props) = edited.nodes[Int(root)] else {
              throw PeleError(code: "test", message: "Expected Gantt instance")
            }
            edited.nodes[Int(root)] = .instance(
              prototype: prototype,
              properties: props.filter { edited.strings[Int($0.key)] != .init("warn") })
            try document.apply(edited, expectedRevision: document.revision)
          }
        }
        let expectedPhase = try XCTUnwrap(row[phase] as? [String: Any])
        var result: Pele.PublicGraph.Atom = .undefined
        do {
          let outcome = try document.invoke(
            method: 28, receiver: document.snapshot().graph.root, arguments: [rawAtom],
            resources: resources)
          switch outcome.result {
          case .returned(let value):
            result = value
            XCTAssertTrue(expectedPhase["error"] is NSNull, id + phase)
          case .thrown(let failure):
            let expectedError = try XCTUnwrap(expectedPhase["error"] as? [String: Any])
            switch failure {
            case .javaScript(let name, let message):
              XCTAssertEqual(name, expectedError["name"] as? String)
              XCTAssertEqual(message.stringIfWellFormed, expectedError["message"] as? String)
            case .pele(let error):
              XCTAssertEqual(try expected(errorJSON(error)), try expected(expectedError))
            default: XCTFail("Unexpected operation failure \(failure)")
            }
          }
        } catch {
          XCTAssertEqual(expectedPhase["error"] as? String, "original", id + phase)
          XCTAssertTrue((error as AnyObject) === original, id + phase)
        }
        let snapshot = try document.snapshot()
        XCTAssertEqual(snapshot.revision, document.revision)
        XCTAssertEqual(
          canonical(snapshot.graph, result: result),
          try expected(XCTUnwrap(expectedPhase["graph"])), id + phase)
        XCTAssertEqual(
          try expected(trace), try expected(XCTUnwrap(expectedPhase["trace"])), id + phase)
        XCTAssertEqual(
          try aliases(snapshot.graph, result, [a, b]), expectedPhase["aliases"] as? [Bool],
          id + phase)
      }
    }
  }
  func testParsedDocumentCalendarResourceReentryPreservesOriginalFailureAndCurrentRevision() throws
  {
    let registry = try Pele.Registry.allBuiltins()
    let document = try registry.parseMutable(
      "gantt\n dateFormat YYYY-MM-DD\n Task :a, 2024-01-01, 1d")
    try registry.close()
    defer { try? document.close() }
    let state = GanttCalendarResourceState(document)
    let argument = try XCTUnwrap(strings(document, ["January 5 2024"]).first)
    var resources = Pele.HostResources()
    resources.timeZone = try XCTUnwrap(TimeZone(secondsFromGMT: 0))
    resources.parseDate = { value in
      try MainActor.assumeIsolated {
        XCTAssertEqual(value, "January 5 2024")
        state.calls += 1
        XCTAssertThrowsError(try state.document.close())
        let before = try state.document.snapshot()
        var graph = before.graph
        let title = try graph.intern("Calendar callback")
        try graph.setField("title", value: .string(title))
        try state.document.apply(graph, expectedRevision: before.revision)
        _ = try state.invoke(14, "Nested")
        let nested = try state.invoke(28, "2024-01-01")
        let current = try state.document.snapshot()
        if case .ref(let id) = nested,
          case .date(let milliseconds, _) = current.graph.nodes[Int(id)]
        {
          XCTAssertEqual(milliseconds, 1_704_067_200_000)
        } else {
          XCTFail("Nested calendar reply lost")
        }
        if state.fail { throw state.failure }
        return 1_704_412_800_000
      }
    }
    let before = document.revision
    XCTAssertThrowsError(
      try document.invoke(
        method: 28, receiver: document.snapshot().graph.root, arguments: [argument],
        resources: resources)
    ) {
      XCTAssertTrue(($0 as AnyObject) === state.failure)
    }
    XCTAssertEqual(state.calls, 1)
    let after = try document.snapshot()
    XCTAssertGreaterThan(after.revision, before)
    XCTAssertEqual(after.revision, document.revision)
    guard case .string(let title) = try after.graph.field("title") else {
      return XCTFail("Callback title lost")
    }
    XCTAssertEqual(after.graph.strings[Int(title)], .init("Calendar callback"))
    state.fail = false
    let recovered = try document.invoke(
      method: 28, receiver: after.graph.root, arguments: [argument], resources: resources)
    guard case .returned(.ref(let result)) = recovered.result,
      case .date(let milliseconds, _) = recovered.graph.nodes[Int(result)]
    else { return XCTFail("Calendar recovery failed") }
    XCTAssertEqual(milliseconds, 1_704_412_800_000)
    XCTAssertEqual(state.calls, 2)
    XCTAssertEqual(recovered.revision, document.revision)
  }
  private func errorJSON(_ error: CompatibilityError) -> [String: Any] {
    [
      "name": error.name, "message": error.message, "code": error.code, "type": error.type ?? "",
      "line": error.line, "column": error.column, "snippet": error.snippet,
    ]
  }
}
