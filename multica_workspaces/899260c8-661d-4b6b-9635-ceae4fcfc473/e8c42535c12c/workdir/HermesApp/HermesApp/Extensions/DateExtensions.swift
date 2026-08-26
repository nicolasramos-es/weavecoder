// DateExtensions.swift
// Date formatting extensions for the app.

import Foundation

extension Date {
    /// Format as "HH:mm" for message timestamps.
    func formattedTime() -> String {
        let formatter = DateFormatter()
        formatter.dateFormat = "HH:mm"
        return formatter.string(from: self)
    }
    
    /// Format as relative time ("2 min ago", "1 hour ago", etc.).
    func formattedRelative() -> String {
        let formatter = RelativeDateTimeFormatter()
        formatter.unitsStyle = .short
        return formatter.localizedString(for: self, relativeTo: Date())
    }
    
    /// Format as ISO 8601 string.
    func formattedISO8601() -> String {
        let formatter = ISO8601DateFormatter()
        return formatter.string(from: self)
    }
}
