import Foundation
import Pele
import XCTest

private final class DirectCalendarError: Error {}
@MainActor private final class DirectCalendarState {
  let document: Pele.MutableDocument
  let failure = DirectCalendarError()
  var fail = true
  var entered = false
  var calls = 0
  var nestedDates: [UInt32] = []
  init(_ document: Pele.MutableDocument) { self.document = document }
  func enter(_ work: Double) throws {
    if entered { return }
    entered = true
    calls += 1
    XCTAssertThrowsError(try document.close())
    let before = try document.snapshot()
    var graph = before.graph
    let title = try graph.intern("Calendar callback")
    let section = try graph.intern("Nested")
    try graph.setField("title", value: .string(title))
    try graph.setField("work", value: .number(work))
    try document.apply(graph, expectedRevision: before.revision)
    let sectionResult = try document.invoke(method: 14, receiver: graph.root, arguments: [.string(section)])
    guard case .returned = sectionResult.result else { throw PeleError(code: "test", message: "Nested section failed") }
    var resources = Pele.HostResources()
    resources.timeZone = TimeZone(secondsFromGMT: 0)!
    let nested = try document.invoke(method: 23, receiver: graph.root, resources: resources)
    guard case .returned(.ref(let id)) = nested.result,
      case .date(let milliseconds, _) = nested.graph.nodes[Int(id)]
    else { throw PeleError(code: "test", message: "Nested today failed") }
    XCTAssertEqual(milliseconds, 1_704_758_400_000)
    nestedDates.append(id)
    if fail { throw failure }
  }
}

@MainActor final class GanttDirectCalendarTests: XCTestCase {
  private func document() throws -> Pele.MutableDocument {
    let registry = try Pele.Registry.allBuiltins()
    let factory = try XCTUnwrap(registry.captureConstructor(for: "gantt", prototype: 1))
    try registry.close()
    let document = try factory.construct()
    factory.close()
    return document
  }
  private func prepare(_ document: Pele.MutableDocument, _ args: [String], now: Double, work: Double) throws -> [Pele.PublicGraph.Atom] {
    var graph = try document.snapshot().graph
    try graph.setField("now", value: .number(now))
    try graph.setField("work", value: .number(work))
    let arguments = try args.map { Pele.PublicGraph.Atom.string(try graph.intern($0)) }
    try document.apply(graph, expectedRevision: document.revision)
    return arguments
  }
  private func expected(_ value: Any) throws -> ConfigValue {
    try JSONDecoder().decode(ConfigValue.self, from: JSONSerialization.data(withJSONObject: value, options: [.fragmentsAllowed]))
  }
  private func result(_ value: Pele.PublicGraph.Atom, _ graph: Pele.PublicGraph) -> [String: Any] {
    switch value {
    case .undefined: return ["kind": "undefined"]
    case .ref(let id):
      guard case .date(let milliseconds, _) = graph.nodes[Int(id)] else { XCTFail("Expected Date"); return [:] }
      return ["kind": "date", "bits": String(format: "%016llx", milliseconds.bitPattern)]
    default: XCTFail("Expected Date/undefined"); return [:]
    }
  }
  func testDirectParseAndTodayMatchPinnedDateBitsWorkAndLimit() throws {
    var path = URL(fileURLWithPath: #filePath)
    for _ in 0..<5 { path.deleteLastPathComponent() }
    let fixture = try XCTUnwrap(JSONSerialization.jsonObject(with: Data(contentsOf: path.appendingPathComponent("fixtures/public-gantt-direct-calendar.json"))) as? [String: Any])
    XCTAssertEqual(fixture["oracleSHA256"] as? String, "d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d")
    let rows = try XCTUnwrap(fixture["cases"] as? [[String: Any]])
    XCTAssertEqual(rows.count, 8)
    for row in rows {
      let id = try XCTUnwrap(row["id"] as? String)
      let method = try XCTUnwrap(row["method"] as? UInt32)
      let doc = try document()
      defer { try? doc.close() }
      let args = try prepare(doc, XCTUnwrap(row["args"] as? [String]), now: (row["now"] as? Double) ?? .nan, work: id == "parse-limit" ? 400_000_000 : 0)
      var resources = Pele.HostResources()
      resources.timeZone = TimeZone(secondsFromGMT: 0)!
      let outcome = try doc.invoke(method: method, receiver: doc.snapshot().graph.root, arguments: args, resources: resources)
      var actualError: Any = NSNull()
      var value: Pele.PublicGraph.Atom = .undefined
      switch outcome.result {
      case .returned(let atom): value = atom
      case .thrown(.pele(let error)):
        actualError = ["name": error.name, "message": error.message, "code": error.code, "type": error.type ?? "", "line": error.line, "column": error.column, "snippet": error.snippet]
      case .thrown(let error): XCTFail("Unexpected error \(error)")
      }
      XCTAssertEqual(try expected(result(value, outcome.graph)), try expected(XCTUnwrap(row["result"])), id)
      XCTAssertEqual(try expected(actualError), try expected(XCTUnwrap(row["error"])), id)
      guard case .number(let work) = try outcome.graph.field("work") else { return XCTFail("Work missing") }
      XCTAssertEqual(work, row["work"] as? Double, id)
    }
  }
  nonisolated private static func fields(_ milliseconds: Double) -> Pele.CalendarFields? {
    guard milliseconds.isFinite else { return nil }
    var calendar = Calendar(identifier: .gregorian)
    calendar.timeZone = TimeZone(secondsFromGMT: 0)!
    let date = Date(timeIntervalSince1970: milliseconds / 1000)
    let c = calendar.dateComponents([.year,.month,.day,.hour,.minute,.second,.nanosecond,.weekday], from: date)
    return .init(civil: .init(year: c.year!, month: c.month!, day: c.day!, hour: c.hour!, minute: c.minute!, second: c.second!, millisecond: c.nanosecond! / 1_000_000), weekday: c.weekday! - 1, utcOffsetMinutes: 0)
  }
  nonisolated private static func instant(_ civil: Pele.CivilDateTime) -> Double? {
    var calendar = Calendar(identifier: .gregorian)
    calendar.timeZone = TimeZone(secondsFromGMT: 0)!
    let c = DateComponents(year: civil.year, month: civil.month, day: civil.day, hour: civil.hour, minute: civil.minute, second: civil.second, nanosecond: civil.millisecond * 1_000_000)
    return calendar.date(from: c).map { ($0.timeIntervalSince1970 * 1000).rounded() }
  }
  func testDirectCalendarCallbacksReenterThrowOriginalAndRecoverWithoutLosingDates() throws {
    for method: UInt32 in [22,23] { for atInstant in [false,true] {
      let doc = try document()
      defer { try? doc.close() }
      let args = try prepare(doc, method == 22 ? ["2024-01-09", "YYYY-MM-DD"] : [], now: 1_704_813_630_000, work: 0)
      let state = DirectCalendarState(doc)
      var resources = Pele.HostResources()
      resources.calendarFields = { value in
        try MainActor.assumeIsolated { if !atInstant { try state.enter(17) } }
        return Self.fields(value)
      }
      resources.calendarInstant = { value in
        try MainActor.assumeIsolated { if atInstant { try state.enter(13) } }
        return Self.instant(value)
      }
      let revision = doc.revision
      XCTAssertThrowsError(try doc.invoke(method: method, receiver: doc.snapshot().graph.root, arguments: args, resources: resources)) {
        XCTAssertTrue(($0 as AnyObject) === state.failure)
      }
      XCTAssertTrue(state.entered)
      XCTAssertEqual(state.calls, 1)
      XCTAssertGreaterThan(doc.revision, revision)
      state.entered = false
      state.fail = false
      let recovered = try doc.invoke(method: method, receiver: doc.snapshot().graph.root, arguments: args, resources: resources)
      XCTAssertEqual(state.calls, 2)
      XCTAssertEqual(recovered.revision, doc.revision)
      guard case .returned(.ref(let returned)) = recovered.result,
        case .date(let milliseconds, _) = recovered.graph.nodes[Int(returned)]
      else { return XCTFail("Recovery Date missing") }
      XCTAssertEqual(milliseconds, 1_704_758_400_000)
      XCTAssertFalse(state.nestedDates.contains(returned))
      XCTAssertEqual(Set(state.nestedDates).count, 2)
      for id in state.nestedDates {
        guard case .date(let nested, _) = recovered.graph.nodes[Int(id)] else { return XCTFail("Nested Date lost") }
        XCTAssertEqual(nested, milliseconds)
      }
      guard case .string(let title) = try recovered.graph.field("title"),
        case .number(let work) = try recovered.graph.field("work")
      else { return XCTFail("Callback fields missing") }
      XCTAssertEqual(recovered.graph.strings[Int(title)], .init("Calendar callback"))
      XCTAssertEqual(work, atInstant ? 13 : 17)
    }}
  }
}
