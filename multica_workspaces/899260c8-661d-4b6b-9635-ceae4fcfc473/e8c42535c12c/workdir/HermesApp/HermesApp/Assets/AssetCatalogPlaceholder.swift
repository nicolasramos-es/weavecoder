// AssetCatalogPlaceholder.swift
// Placeholder for asset catalog configuration.
// In a real Xcode project, this would reference Assets.xcassets.
// For SPM, we use system images and colors.

import SwiftUI

// MARK: - App Icons (System Images)

enum AppIcons {
    static let appIcon = "message.badge.fill"
    static let chatIcon = "bubble.right"
    static let chatsIcon = "message"
    static let settingsIcon = "gear"
    static let serverIcon = "server.rack"
    static let sendIcon = "paperplane.fill"
    static let commandsIcon = "slash.circle"
    static let newChatIcon = "square.and.pencil"
    static let deleteIcon = "trash"
    static let clearIcon = "arrow.counterclockwise"
    static let statusIcon = "ellipsis.circle"
}

// MARK: - App Colors

enum AppColors {
    static let primary = Color.blue
    static let secondary = Color.gray
    static let userBubble = Color.blue
    static let assistantBubble = Color(.systemGray6)
    static let error = Color.red
    static let warning = Color.orange
    static let success = Color.green
}
