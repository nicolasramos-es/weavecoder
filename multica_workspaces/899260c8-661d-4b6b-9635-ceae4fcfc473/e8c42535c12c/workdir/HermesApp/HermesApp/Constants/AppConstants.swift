// AppConstants.swift
// App-wide constants and configuration.

import Foundation

enum AppConstants {
    // MARK: - API Configuration
    
    enum API {
        static let defaultBaseURL = "http://localhost:8642"
        static let defaultModel = "default"
        static let requestTimeout: TimeInterval = 60.0
        static let resourceTimeout: TimeInterval = 600.0
        
        // MARK: - Endpoint Paths
        
        enum Endpoints {
            static let health = "health"
            static let v1Models = "v1/models"
            static let v1ChatCompletions = "v1/chat/completions"
            static let apiSessions = "api/sessions"
            static let apiSessionsList = "api/sessions/list"
            static let apiSessionsNew = "api/sessions/new"
            static let apiSessionsHistory = "api/sessions/history"
            static let apiSessionsClear = "api/sessions/clear"
            static let apiSessionsDelete = "api/sessions/delete"
            static let apiCommands = "api/commands"
            static let apiStatus = "api/status"
            static let apiNotifications = "api/notifications"
            static let apiNotificationsRegister = "api/notifications/register"
        }
    }
    
    // MARK: - UserDefaults Keys
    
    enum UserDefaultsKeys {
        static let sessions = "hermes_sessions"
        static let apiBaseUrl = "apiBaseUrl"
        static let apiKey = "apiKey"
        static let defaultModel = "defaultModel"
        static let enableStreaming = "enableStreaming"
        static let enableNotifications = "enableNotifications"
        static let messageTimestamps = "messageTimestamps"
    }
    
    // MARK: - UI Constants
    
    enum UI {
        static let bubbleCornerRadius: CGFloat = 18
        static let bubbleHorizontalPadding: CGFloat = 12
        static let bubbleVerticalPadding: CGFloat = 8
        static let loadingDotSize: CGFloat = 8
        static let sendButtonSize: CGFloat = 40
    }
    
    // MARK: - App Information
    
    enum App {
        static let name = "Hermes"
        static let version = "0.1.0"
        static let platform = "Hermes Agent"
    }
}
