// ChatView.swift
// Main chat interface with message display and input.

import SwiftUI

// MARK: - ChatView

@available(iOS 17.0, *)
struct ChatView: View {
    @EnvironmentObject var chatViewModel: ChatViewModel
    @EnvironmentObject var sessionManager: SessionManager
    @State private var messageInput = ""
    @State private var showCommandPicker = false
    @FocusState private var isInputFocused: Bool
    
    var body: some View {
        VStack(spacing: 0) {
            // Header
            chatHeader
            
            // Messages list
            messagesList
            
            // Input area
            inputArea
            
            // Command picker
            if showCommandPicker {
                commandPickerSheet
            }
        }
        .onAppear {
            chatViewModel.loadMessages()
        }
        .alert("Error", isPresented: $chatViewModel.showError) {
            Button("OK") {}
        } message: {
            Text(chatViewModel.errorMessage ?? "Unknown error")
        }
    }
    
    // MARK: - Header
    
    private var chatHeader: some View {
        HStack {
            if let session = chatViewModel.currentSession {
                Text(session.title)
                    .font(.headline)
            } else {
                Text("Hermes Bot")
                    .font(.headline)
            }
            
            Spacer()
            
            // Session controls
            Menu {
                Button("New Session") {
                    chatViewModel.createNewSession()
                }
                Button("Clear History") {
                    chatViewModel.clearCurrentSession()
                }
                Button("Delete Session") {
                    chatViewModel.deleteCurrentSession()
                }
                Divider()
                Button("Status") {
                    chatViewModel.checkServerStatus()
                }
            } label: {
                Image(systemName: "ellipsis.circle")
                    .font(.title3)
            }
        }
        .padding(.horizontal)
        .padding(.vertical, 8)
        .background(Color(.systemGray6))
    }
    
    // MARK: - Messages List
    
    private var messagesList: some View {
        ScrollViewReader { proxy in
            ScrollView {
                LazyVStack(spacing: 8) {
                    ForEach(chatViewModel.messages) { message in
                        MessageBubble(message: message)
                            .id(message.id)
                    }
                    
                    if chatViewModel.isSending {
                        LoadingIndicator()
                    }
                }
                .padding(.horizontal)
                .padding(.vertical, 8)
            }
            .onChange(of: chatViewModel.messages.count) {
                withAnimation {
                    proxy.scrollTo(chatViewModel.messages.last?.id, anchor: .bottom)
                }
            }
        }
    }
    
    // MARK: - Input Area
    
    private var inputArea: some View {
        VStack(spacing: 0) {
            Divider()
            
            HStack(spacing: 8) {
                // Command button
                Button {
                    showCommandPicker.toggle()
                } label: {
                    Image(systemName: "slash.circle")
                        .font(.title2)
                        .foregroundColor(.blue)
                }
                
                // Text input
                TextField("Message Hermes...", text: $messageInput)
                    .textFieldStyle(.roundedBorder)
                    .focused($isInputFocused)
                    .onSubmit {
                        sendMessage()
                    }
                
                // Send button
                Button {
                    sendMessage()
                } label: {
                    Image(systemName: "paperplane.fill")
                        .font(.title2)
                        .foregroundColor(.white)
                        .frame(width: 40, height: 40)
                        .background(Color.blue)
                        .clipShape(Circle())
                }
                .disabled(messageInput.trimmingCharacters(in: .whitespaces).isEmpty || chatViewModel.isSending)
            }
            .padding(.horizontal)
            .padding(.vertical, 8)
        }
        .background(Color(.systemGray6))
    }
    
    // MARK: - Command Picker
    
    private var commandPickerSheet: some View {
        NavigationStack {
            List(chatViewModel.availableCommands, id: \.id) { command in
                VStack(alignment: .leading, spacing: 4) {
                    Text(command.name)
                        .font(.headline)
                    Text(command.description)
                        .font(.subheadline)
                        .foregroundColor(.secondary)
                }
                .onTapGesture {
                    messageInput = command.name + " "
                    showCommandPicker = false
                    isInputFocused = true
                }
            }
            .navigationTitle("Commands")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button("Done") {
                        showCommandPicker = false
                    }
                }
            }
        }
    }
    
    // MARK: - Actions
    
    private func sendMessage() {
        let text = messageInput.trimmingCharacters(in: .whitespaces)
        guard !text.isEmpty else { return }
        
        chatViewModel.sendMessage(text)
        messageInput = ""
        isInputFocused = false
    }
}

// MARK: - Message Bubble

struct MessageBubble: View {
    let message: ChatMessage
    
    var body: some View {
        HStack {
            if message.role == .user {
                Spacer()
                userBubble
            } else {
                assistantBubble
                Spacer()
            }
        }
    }
    
    private var userBubble: some View {
        VStack(alignment: .trailing, spacing: 4) {
            Text(message.content)
                .font(.body)
                .foregroundColor(.white)
                .padding(.horizontal, 12)
                .padding(.vertical, 8)
                .background(Color.blue)
                .cornerRadius(18)
            
            Text(formatTimestamp(message.timestamp))
                .font(.caption2)
                .foregroundColor(.secondary)
        }
        .padding(.horizontal, 4)
    }
    
    private var assistantBubble: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(message.content)
                .font(.body)
                .foregroundColor(.primary)
                .padding(.horizontal, 12)
                .padding(.vertical, 8)
                .background(Color(.systemGray6))
                .cornerRadius(18)
            
            Text(formatTimestamp(message.timestamp))
                .font(.caption2)
                .foregroundColor(.secondary)
        }
        .padding(.horizontal, 4)
    }
    
    private func formatTimestamp(_ date: Date) -> String {
        let formatter = DateFormatter()
        formatter.dateFormat = "HH:mm"
        return formatter.string(from: date)
    }
}

// MARK: - Loading Indicator

struct LoadingIndicator: View {
    var body: some View {
        HStack {
            assistantBubble
            Spacer()
        }
        .padding(.horizontal)
    }
    
    private var assistantBubble: some View {
        HStack(spacing: 4) {
            Circle()
                .fill(Color.gray.opacity(0.4))
                .frame(width: 8, height: 8)
                .scaleEffect(1.0)
                .animation(.easeInOut.repeatForever(autoreverses: true), value: Date())
            
            Circle()
                .fill(Color.gray.opacity(0.4))
                .frame(width: 8, height: 8)
                .scaleEffect(1.0)
                .animation(.easeInOut.repeatForever(autoreverses: true, delay: 0.2), value: Date())
            
            Circle()
                .fill(Color.gray.opacity(0.4))
                .frame(width: 8, height: 8)
                .scaleEffect(1.0)
                .animation(.easeInOut.repeatForever(autoreverses: true, delay: 0.4), value: Date())
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 8)
        .background(Color(.systemGray6))
        .cornerRadius(18)
    }
}

// MARK: - Preview

#if DEBUG
@available(iOS 17.0, *)
struct ChatView_Previews: PreviewProvider {
    static var previews: some View {
        ChatView()
            .environmentObject(ChatViewModel(client: HermesClient(config: .init(baseURL: URL(string: "http://localhost:8642")!, apiKey: "test"))))
            .environmentObject(SessionManager(client: HermesClient(config: .init(baseURL: URL(string: "http://localhost:8642")!, apiKey: "test"))))
    }
}
#endif
