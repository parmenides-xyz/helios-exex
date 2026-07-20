import SwiftUI

@main
struct DataNetworkDemoApp: App {
    var body: some Scene {
        WindowGroup {
            DataNetworkNodeView()
                .preferredColorScheme(.dark)
        }
    }
}
