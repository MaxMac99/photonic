import XCTest

final class PhotonicUITests: XCTestCase {
    func testLaunchShowsRootTabs() {
        let app = XCUIApplication()
        app.launch()

        XCTAssertTrue(app.tabBars.firstMatch.waitForExistence(timeout: 5))
    }
}
