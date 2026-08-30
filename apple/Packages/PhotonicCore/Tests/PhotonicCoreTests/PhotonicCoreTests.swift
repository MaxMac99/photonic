import Foundation
import Testing
@testable import PhotonicCore

struct ServerURLTests {
    @Test(arguments: [
        "http://localhost:8080",
        "https://photonic.example.com",
        "https://photonic.example.com/base/path"
    ])
    func acceptsValidBaseURLs(_ string: String) {
        #expect(ServerURL(string) != nil)
    }

    @Test(arguments: [
        "",
        "photonic.example.com",
        "ftp://photonic.example.com",
        "https://",
        "https://photonic.example.com?token=abc",
        "https://photonic.example.com#section",
    ])
    func rejectsInvalidURLs(_ string: String) {
        #expect(ServerURL(string) == nil)
    }

    @Test
    func codableRoundTrip() throws {
        let original = try #require(ServerURL("https://photonic.example.com"))
        let data = try JSONEncoder().encode(original)
        let decoded = try JSONDecoder().decode(ServerURL.self, from: data)
        #expect(decoded == original)
    }

    @Test
    func decodingInvalidValueThrows() throws {
        let data = try JSONEncoder().encode("not a url")
        #expect(throws: DecodingError.self) {
            try JSONDecoder().decode(ServerURL.self, from: data)
        }
    }
}

struct AccessTokenTests {
    @Test
    func acceptsNonEmptyToken() throws {
        let token = try #require(AccessToken(value: "abc123", expiresAt: .distantFuture))
        #expect(!token.isExpired(at: .distantPast))
    }

    @Test(arguments: ["", "   "])
    func rejectsBlankToken(_ value: String) {
        #expect(AccessToken(value: value, expiresAt: .distantFuture) == nil)
    }

    @Test
    func expiryBoundary() throws {
        let expiry = Date(timeIntervalSince1970: 1000)
        let token = try #require(AccessToken(value: "abc", expiresAt: expiry))
        #expect(!token.isExpired(at: Date(timeIntervalSince1970: 999)))
        #expect(token.isExpired(at: expiry))
        #expect(token.isExpired(at: Date(timeIntervalSince1970: 1001)))
    }
}

struct RefreshTokenTests {
    @Test
    func acceptsNonEmptyToken() {
        #expect(RefreshToken(value: "abc123") != nil)
    }

    @Test(arguments: ["", "   "])
    func rejectsBlankToken(_ value: String) {
        #expect(RefreshToken(value: value) == nil)
    }
}

struct MediaHashTests {
    @Test(arguments: [
        "0123456789abcdef0123456789abcdef",
        "ABCDEF0123456789ABCDEF0123456789",
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
    ])
    func acceptsValidHashes(_ value: String) throws {
        let hash = try #require(MediaHash(value))
        #expect(hash.value == value.lowercased())
    }

    @Test(arguments: [
        "",
        "0123456789abcdeg0123456789abcdef", // non-hex
        "01234567", // too short
        String(repeating: "0123456789abcdef", count: 8) + "0" // too long (129)
    ])
    func rejectsInvalidHashes(_ value: String) {
        #expect(MediaHash(value) == nil)
    }

    @Test
    func decodingInvalidValueThrows() throws {
        let data = try JSONEncoder().encode("nothex")
        #expect(throws: DecodingError.self) {
            try JSONDecoder().decode(MediaHash.self, from: data)
        }
    }
}
