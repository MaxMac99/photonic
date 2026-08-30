import Foundation
import Testing
@testable import PhotonicAPI

struct PhotonicAPITests {
    @Test
    func clientCanBeCreated() throws {
        let serverURL = try #require(URL(string: "http://localhost:8080"))
        _ = APIClientFactory.make(serverURL: serverURL)
    }
}
