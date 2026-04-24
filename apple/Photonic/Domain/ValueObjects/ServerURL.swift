//
//  ServerURL.swift
//  Photonic
//
//  Domain Value Object
//

import Foundation

struct ServerURL: Equatable, Hashable {
    let value: URL

    init?(string: String) {
        guard let url = URL(string: string),
              let scheme = url.scheme,
              ["http", "https"].contains(scheme.lowercased()),
              url.host != nil
        else {
            return nil
        }
        value = url
    }

    init?(url: URL) {
        guard let scheme = url.scheme,
              ["http", "https"].contains(scheme.lowercased()),
              url.host != nil
        else {
            return nil
        }
        value = url
    }

    var isSecure: Bool {
        value.scheme?.lowercased() == "https"
    }

    var host: String? {
        value.host
    }

    var absoluteString: String {
        value.absoluteString
    }
}
