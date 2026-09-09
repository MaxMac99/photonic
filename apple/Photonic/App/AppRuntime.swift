import Foundation

/// Runtime detection for test execution. Unit tests launch the real app as
/// their test host; the app must stay inert (no onAppear side effects) so
/// its live dependency accesses don't pollute test runs.
enum AppRuntime {
    static let isRunningUnitTests =
        ProcessInfo.processInfo.environment["XCTestConfigurationFilePath"] != nil
            || ProcessInfo.processInfo.arguments.contains("--unit-tests")
}
