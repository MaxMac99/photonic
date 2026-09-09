import Dependencies
import Foundation
import PhotonicCore

extension ServerConfigurationClient: DependencyKey {
    static var liveValue: ServerConfigurationClient {
        makeClient(defaults: .standard)
    }

    /// Non-persisting store for previews.
    static var inMemoryValue: ServerConfigurationClient {
        let store = MemoryStore()
        return ServerConfigurationClient(
            load: { await store.load() },
            save: { await store.save($0) },
            clear: { await store.clear() }
        )
    }

    private static let storageKey = "serverConfiguration"

    private static func makeClient(defaults: UserDefaults) -> ServerConfigurationClient {
        ServerConfigurationClient(
            load: {
                guard let data = defaults.data(forKey: storageKey) else { return nil }
                return try? JSONDecoder().decode(ServerConfiguration.self, from: data)
            },
            save: { configuration in
                let data = try JSONEncoder().encode(configuration)
                defaults.set(data, forKey: storageKey)
            },
            clear: { defaults.removeObject(forKey: storageKey) }
        )
    }
}

private actor MemoryStore {
    private var value: ServerConfiguration?

    func load() -> ServerConfiguration? {
        value
    }

    func save(_ configuration: ServerConfiguration) {
        value = configuration
    }

    func clear() {
        value = nil
    }
}
