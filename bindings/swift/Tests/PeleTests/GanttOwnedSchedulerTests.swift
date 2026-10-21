import Foundation
import Pele
import XCTest

private final class GanttSchedulerError: Error {}
private final class GanttSchedulerHost: Pele.PublicHostCallable {
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

@MainActor final class GanttOwnedSchedulerTests: XCTestCase {
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

  private func array(_ graph: inout Pele.PublicGraph, _ values: [Pele.PublicGraph.Atom]) throws -> UInt32 {
    let id = UInt32(graph.nodes.count)
    graph.nodes.append(.array(length: 0, properties: []))
    for (index, value) in values.enumerated() { try graph.setArrayElement(UInt32(index), value: value, on: id) }
    return id
  }
  func testOwnedSchedulerMatches20OracleWarningReentryDateAndTaskIteratorCases() throws {
    var path = URL(fileURLWithPath: #filePath)
    for _ in 0..<5 { path.deleteLastPathComponent() }
    let fixture = try XCTUnwrap(JSONSerialization.jsonObject(with: Data(contentsOf: path.appendingPathComponent("fixtures/public-gantt-owned-scheduler.json"))) as? [String: Any])
    XCTAssertEqual(fixture["oracleSHA256"] as? String, "d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d")
    let rows = try XCTUnwrap(fixture["cases"] as? [[String: Any]])
    XCTAssertEqual(rows.count, 20)
    for row in rows {
      let id = try XCTUnwrap(row["id"] as? String)
      let mode = try XCTUnwrap(row["mode"] as? String)
      let method = UInt32(try XCTUnwrap(row["method"] as? Int))
      let registry = try Pele.Registry.allBuiltins()
      let options = GanttSchedulerHost(), warning = GanttSchedulerHost()
      let factory = try XCTUnwrap(registry.captureConstructor(for: "gantt", prototype: 1))
      let document = try factory.construct(firstArgument: .host(options))
      factory.close(); try registry.close()
      defer { try? document.close() }
      var graph = try document.snapshot().graph
      let format = try graph.intern("YYYY-MM-DD")
      try graph.setField("dateFormat", value: .string(format))
      try graph.setField("now", value: .number(1_704_813_630_000))
      try document.apply(graph, expectedRevision: document.revision)
      try addTask(document, "Alpha", mode == "excludes" ? ":a,2024-01-01,3d" : ":a,2024-01-01,bad-duration")
      try addTask(document, "Beta", ":b,after a,1d")
      graph = try document.snapshot().graph
      let tasks = try ref(graph.field("tasks")), originalA = try ref(graph.field("0", on: tasks))
      if mode == "excludes" {
        let date = try graph.intern("2024-01-02")
        let excluded = try array(&graph, [.string(date)])
        try graph.setField("excludes", value: .ref(excluded))
        let set = UInt32(graph.nodes.count)
        graph.nodes.append(.set(values: [.string(date)], properties: []))
        try graph.setField("excludeSet", value: .ref(set))
        let days = try array(&graph, Array(repeating: .bool(false), count: 7))
        try graph.setField("excludedDays", value: .ref(days))
        try graph.setField("datedTokens", value: .bool(true))
        try document.apply(graph, expectedRevision: document.revision)
      }
      let original = GanttSchedulerError()
      var trace: [[Any]] = [], reads = 0
      options.get = { key in
        XCTAssertEqual(key, .init("warn"), id)
        reads += 1; trace.append(["get", reads])
        return mode == "warn-false" ? .bool(false) : .host(warning)
      }
      warning.call = { receiver, arguments in
        guard case .host(let owner) = receiver, case .string(let message) = arguments.first else { throw PeleError(code: "test", message: "Invalid warning call") }
        XCTAssertTrue(owner === options, id)
        trace.append(["call", owner === options, message.codeUnits.map(Int.init)])
        XCTAssertThrowsError(try document.close())
        if mode == "warn-reentry" {
          _ = try self.invoke(document, 14, self.strings(document, ["Nested"]))
          var edited = try document.snapshot().graph
          let task = try self.firstTask(edited), raw = try self.ref(edited.field("raw", on: task))
          let end = try edited.intern("2d")
          try edited.setField("end", value: .string(end), on: raw)
          let currentTasks = try self.ref(edited.field("tasks"))
          guard case .array(let count, _) = edited.nodes[Int(currentTasks)] else { throw GanttSchedulerError() }
          try document.apply(edited, expectedRevision: document.revision)
          if count == 2 { try self.addTask(document, "Appended", ":c,2024-01-05,1d") }
        }
        if mode == "warn-throw" { throw original }
        return .undefined
      }
      var resources = Pele.HostResources()
      resources.timeZone = try XCTUnwrap(TimeZone(secondsFromGMT: 0))
      for phase in ["first", "recovery"] {
        if phase == "recovery" { options.get = { _ in .undefined }; trace.removeAll() }
        let expectedPhase = try XCTUnwrap(row[phase] as? [String: Any])
        var args: [Pele.PublicGraph.Atom]
        switch method {
        case 19: args = [.number(1_704_813_630_000)]
        case 25: args = []
        case 26: args = [.ref(originalA)]
        case 30:
          var current = try document.snapshot().graph
          let duration = try current.intern(mode == "excludes" ? "3d" : "bad-duration")
          let date = UInt32(current.nodes.count)
          current.nodes.append(.date(milliseconds: 1_704_067_200_000, properties: []))
          try document.apply(current, expectedRevision: document.revision)
          args = [.ref(date), .string(duration)]
        default: throw PeleError(code: "test", message: "Unexpected scheduler method")
        }
        var result: Pele.PublicGraph.Atom = .undefined
        do {
          let outcome = try document.invoke(method: method, receiver: document.snapshot().graph.root, arguments: args, resources: resources)
          guard case .returned(let value) = outcome.result else { return XCTFail("Unexpected scheduler error: \(id)/\(phase)/\(outcome.result)") }
          result = value
          XCTAssertTrue(expectedPhase["error"] is NSNull, id + phase)
        } catch {
          XCTAssertEqual(expectedPhase["error"] as? String, "original", id + phase)
          XCTAssertTrue((error as AnyObject) === original, id + phase)
        }
        let snapshot = try document.snapshot()
        XCTAssertEqual(snapshot.revision, document.revision)
        XCTAssertEqual(canonical(snapshot.graph, result: result), try expected(XCTUnwrap(expectedPhase["graph"])), id + phase)
        XCTAssertEqual(try expected(trace), try expected(XCTUnwrap(expectedPhase["trace"])), id + phase)
        let returnedTasks: Bool
        if case .ref(let value) = result { returnedTasks = value == tasks } else { returnedTasks = false }
        XCTAssertEqual(returnedTasks, expectedPhase["returnedTasks"] as? Bool, id + phase)
      }
      if mode == "warn-reentry" { XCTAssertGreaterThan(reads, 0, id + "/callback reached") }
    }
  }
}
