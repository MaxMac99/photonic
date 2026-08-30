import Dependencies

/// The composition root wires live implementations to the typed dependency
/// clients every feature declares. Feature code never imports an adapter.
enum CompositionRoot {
    /// Registers the live dependency values. Called once at app start,
    /// before any store is created.
    static func registerLiveDependencies() {
        Dependencies.prepareDependencies { _ in
            // Live adapters register here as they land:
            // - PhotonicAPI (build-order step 2)
            // - durable backup queue + background session (step 4)
            // - auth + photo library clients (step 5)
        }
    }
}
