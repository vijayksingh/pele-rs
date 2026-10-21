import Foundation
import XCTest
import Pele

final class DirectConstructorTests: XCTestCase {
    private struct Fixture: Decodable {
        struct Case: Decodable {
            struct State: Decodable {
                struct Drawing: Decodable {
                    let ok: Bool
                    let svg: String?
                    let width: String?
                    let height: String?
                    let error: CompatibilityError?
                }
                let render: Drawing
            }
            let family: String
            let prototype: UInt32
            let initial: State
        }
        let oracleSHA256: String
        let cases: [Case]
    }

    func testFifteenCapturedConstructorsUseNativeDefaultsAndRawRenderingAfterClosure() throws {
        var path = URL(fileURLWithPath: #filePath)
        for _ in 0..<5 { path.deleteLastPathComponent() }
        let files = ["public-direct-constructors.json", "public-next-six-constructors.json", "public-state-constructor.json", "public-class-constructor.json"]
        let fixtures = try files.map {
            try JSONDecoder().decode(Fixture.self, from: Data(contentsOf: path.appendingPathComponent("fixtures/" + $0)))
        }
        for fixture in fixtures {
            XCTAssertEqual(fixture.oracleSHA256, "d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d")
        }
        let cases = fixtures.flatMap(\.cases)
        XCTAssertEqual(cases.map(\.family).sorted(),
                       ["architecture", "c4", "class", "er", "ishikawa", "journey", "kanban", "mindmap", "quadrantChart", "railroad", "requirement", "state", "timeline", "venn", "xychart"])
        let registry = try Pele.Registry.allBuiltins()
        let factories = try cases.map {
            try XCTUnwrap(registry.captureConstructor(for: $0.family, prototype: $0.prototype), $0.family)
        }
        try registry.close()
        var resources = Pele.HostResources()
        resources.textWidth = { Double($0.text.utf16.count) * $0.size * 0.6 }
        var options = Pele.Engine.Options(); options.now = 1_704_067_200_000
        for (row, factory) in zip(cases, factories) {
            let model = try factory.constructPrepared(), sibling = try factory.constructPrepared()
            defer { try? model.close(); try? sibling.close() }
            factory.close()
            XCTAssertEqual(model.type, row.family)
            XCTAssertTrue(model.isConstructorCreated)
            XCTAssertEqual(model.revision, 0)
            XCTAssertEqual(sibling.revision, 0)
            XCTAssertTrue(sibling.isConstructorCreated)
            let snapshot = try model.snapshot()
            guard case .ref(let root) = snapshot.graph.root,
                  case .instance(let prototype, _) = snapshot.graph.nodes[Int(root)] else {
                return XCTFail("\(row.family): constructor must create its real class instance")
            }
            XCTAssertEqual(prototype, row.prototype, row.family)
            let result = try model.renderOperation(options: options, resources: resources)
            switch result.result {
            case .rendered(let drawing):
                XCTAssertTrue(row.initial.render.ok, row.family)
                XCTAssertEqual(drawing.svg, row.initial.render.svg, row.family)
                XCTAssertEqual(String(format: "%016llx", drawing.width.bitPattern), row.initial.render.width, row.family)
                XCTAssertEqual(String(format: "%016llx", drawing.height.bitPattern), row.initial.render.height, row.family)
            case .thrown(.pele(let actual)):
                XCTAssertFalse(row.initial.render.ok, row.family)
                let expected = try XCTUnwrap(row.initial.render.error, row.family)
                XCTAssertEqual(actual.name, expected.name, row.family)
                XCTAssertEqual(actual.message, expected.message, row.family)
                XCTAssertEqual(actual.code, expected.code, row.family)
                XCTAssertEqual(actual.type, expected.type, row.family)
                XCTAssertEqual(actual.line, expected.line, row.family)
                XCTAssertEqual(actual.column, expected.column, row.family)
                XCTAssertEqual(actual.snippet, expected.snippet, row.family)
            case .thrown:
                XCTFail("\(row.family): unexpected error category")
            }
            XCTAssertEqual(model.revision, result.revision, row.family)
            XCTAssertGreaterThan(model.revision, 0, row.family)
            XCTAssertEqual(sibling.revision, 0, row.family)
            XCTAssertEqual(try sibling.snapshot().type, row.family)
        }
    }
}
