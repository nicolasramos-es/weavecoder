// HermesApp.swift
// Entry point for the Hermes iOS app.

import SwiftUI

@available(iOS 17.0, *)
@main
struct HermesApp: App {
    @StateObject private var sessionManager: SessionManager
    @StateObject private var chatViewModel: ChatViewModel
    
    init() {
        // TODO: Replace with actual API key from secure storage
        let apiKey = ProcessInfo.processInfo.environment["HERMES_API_KEY"] ?? "your-api-key-here"
        
        let config = HermesClient.Config(
            baseURL: URL(string: "http://localhost:8642")!,
            apiKey: apiKey,
            defaultModel: "default"
        )
        
        let client = HermesClient(config: config)
        _sessionManager = StateObject(wrappedValue: SessionManager(client: client))
        _chatViewModel = StateObject(wrappedValue: ChatViewModel(client: client))
    }
    
    var body: some Scene {
        WindowGroup {
            ContentView()
                .environmentObject(sessionManager)
                .environmentObject(chatViewModel)
        }
    }
}

// MARK: - ContentView

@available(iOS 17.0, *)
struct ContentView: View {
    @EnvironmentObject var sessionManager: SessionManager
    @EnvironmentObject var chatViewModel: ChatViewModel
    @State private var selectedTab = 0
    
    var body: some View {
        TabView(selection: $selectedTab) {
            ChatListView()
                .tabItem {
                    Label("Chats", systemImage: "message")
                }
                .tag(0)
                .environmentObject(sessionManager)
            
            ChatView()
                .tabItem {
                    Label("Chat", systemImage: "bubble.right")
                }
                .tag(1)
                .environmentObject(chatViewModel)
            
            SettingsView()
                .tabItem {
                    Label("Settings", systemImage: "gear")
                }
                .tag(2)
        }
    }
}
