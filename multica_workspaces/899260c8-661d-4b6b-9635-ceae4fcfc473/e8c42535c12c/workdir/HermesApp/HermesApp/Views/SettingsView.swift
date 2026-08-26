// SettingsView.swift
// App settings: API configuration, model selection, notifications.

import SwiftUI

// MARK: - SettingsView

@available(iOS 17.0, *)
struct SettingsView: View {
    @AppStorage("apiBaseUrl") private var apiBaseUrl: String = "http://localhost:8642"
    @AppStorage("apiKey") private var apiKey: String = ""
    @AppStorage("defaultModel") private var defaultModel: String = "default"
    @AppStorage("enableStreaming") private var enableStreaming: Bool = true
    @AppStorage("enableNotifications") private var enableNotifications: Bool = false
    @AppStorage("messageTimestamps") private var messageTimestamps: Bool = false
    
    @State private var testConnection = false
    @State private var connectionStatus: String = ""
    @State private var availableModels: [String] = []
    @State private var showingModelPicker = false
    
    var body: some View {
        NavigationStack {
            Form {
                // API Configuration
                Section("API Configuration") {
                    TextField("Base URL", text: $apiBaseUrl)
                        .autocorrectionDisabled()
                    
                    SecureField("API Key", text: $apiKey)
                    
                    TextField("Default Model", text: $defaultModel)
                        .autocorrectionDisabled()
                }
                
                // Connection Test
                Section("Connection") {
                    Button {
                        testConnectionAction()
                    } label: {
                        HStack {
                            if testConnection {
                                ProgressView()
                                    .progressViewStyle(CircularProgressViewStyle())
                            }
                            Text(testConnection ? "Testing..." : "Test Connection")
                        }
                    }
                    .disabled(testConnection)
                    
                    if !connectionStatus.isEmpty {
                        Text(connectionStatus)
                            .font(.caption)
                            .foregroundColor(connectionStatus.hasPrefix("Error") ? .red : .green)
                    }
                }
                
                // Available Models
                if !availableModels.isEmpty {
                    Section("Available Models") {
                        ForEach(availableModels, id: \.self) { model in
                            Text(model)
                                .font(.subheadline)
                        }
                    }
                }
                
                // App Settings
                Section("App Settings") {
                    Toggle("Enable Streaming", isOn: $enableStreaming)
                    
                    Toggle("Enable Notifications", isOn: $enableNotifications)
                        .onChange(of: enableNotifications) { newValue in
                            if newValue {
                                requestNotificationPermission()
                            }
                        }
                    
                    Toggle("Show Message Timestamps", isOn: $messageTimestamps)
                }
                
                // About
                Section("About") {
                    LabeledContent("Platform", value: "Hermes Agent")
                    LabeledContent("Version", value: "0.1.0")
                    LabeledContent("iOS", value: ProcessInfo.processInfo.operatingSystemVersionString)
                }
            }
            .navigationTitle("Settings")
            .onChange(of: apiBaseUrl) { _ in
                saveConfig()
            }
            .onChange(of: apiKey) { _ in
                saveConfig()
            }
            .onChange(of: defaultModel) { _ in
                saveConfig()
            }
        }
    }
    
    // MARK: - Actions
    
    private func testConnectionAction() {
        testConnection = true
        connectionStatus = ""
        
        Task {
            do {
                let config = HermesClient.Config(
                    baseURL: URL(string: apiBaseUrl) ?? URL(string: "http://localhost:8642")!,
                    apiKey: apiKey,
                    defaultModel: defaultModel
                )
                let client = HermesClient(config: config)
                
                let health = try await client.checkHealth()
                connectionStatus = "Connected: \(health.platform) v\(health.version)"
                
                // Fetch available models
                do {
                    let models = try await client.listModels()
                    availableModels = models.data.map { $0.id }
                } catch {
                    availableModels = []
                }
            } catch {
                connectionStatus = "Error: \(error.localizedDescription)"
                availableModels = []
            }
            
            testConnection = false
        }
    }
    
    private func requestNotificationPermission() {
        // TODO: Implement push notification registration
        // This would use UserNotifications framework
        print("Notification permission requested")
    }
    
    private func saveConfig() {
        // Config is saved via @AppStorage automatically
    }
}

// MARK: - Preview

#if DEBUG
@available(iOS 17.0, *)
struct SettingsView_Previews: PreviewProvider {
    static var previews: some View {
        SettingsView()
    }
}
#endif
