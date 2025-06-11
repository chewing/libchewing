//
//  ContentView.swift
//  chewing
//
//  Created by Rumen Cholakov on 2.06.25.
//

import SwiftUI
import Chewing
import os

/// ViewModel that wraps ChewingWrapper and publishes updates.
class ChewingViewModel: ObservableObject {
  private let logger = Logger(subsystem: "ChewingViewModel", category: "")

  @Published var committedText: String = ""
  @Published var preeditText: String = ""
  @Published var bufferText: String = ""
  @Published var candidates: [String] = []
  
  private var chewingWrapper: ChewingWrapper?
  
  init() {
    initializeChewing()
  }
  
  private func initializeChewing() {
    guard let dataPath = ChewingWrapper.dataDirectoryPath else {
      logger.error("Could not locate chewing data directory.")
      return
    }
    do {
      
      let wrapper = try ChewingWrapper(
        candPerPage: 10,
        maxChiSymbolLen: 18,
        dataDirectoryPath: dataPath,
        loggingConfig: .init()
      )
      
      // Assign callbacks to update @Published properties
      wrapper.onCommit = { [weak self] text in
        DispatchQueue.main.async {
          self?.committedText.append(text)
          self?.bufferText = ""
          self?.preeditText = ""
          self?.candidates = []
        }
      }
      wrapper.onBufferUpdate = { [weak self] text in
        DispatchQueue.main.async {
          self?.bufferText = text
        }
      }
      wrapper.onPreeditUpdate = { [weak self] text in
        DispatchQueue.main.async {
          self?.preeditText = text
        }
      }
      wrapper.onCandidateUpdate = { [weak self] candidates in
        DispatchQueue.main.async {
          self?.candidates = candidates
        }
      }
      
      self.chewingWrapper = wrapper
    } catch {
      logger.error("Failed to initialize ChewingWrapper: \(error)")
    }
  }
  
  /// Process a single character input.
  func process(character: Character) {
    chewingWrapper?.process(key: character)
  }
  /// Process a single character input.
  func process(character: ChewingKey) {
    chewingWrapper?.process(key: character)
  }
  
  /// Select a candidate by index.
  func selectCandidate(at index: Int) {
    chewingWrapper?.selectCandidate(at: index)
  }
}

struct ContentView: View {
  @StateObject private var viewModel = ChewingViewModel()
  @State private var userInput: String = ""
  
  var body: some View {
    VStack(spacing: 16) {
      Text("Data Path: \(ChewingWrapper.dataDirectoryPath ?? "Not found")")
        .font(.footnote)
        .foregroundColor(.gray)
      
      // Show buffer, preedit, committed, and candidates
      VStack(alignment: .leading, spacing: 8) {
        Text("Committed: \(viewModel.committedText)")
        Text("Preedit: \(viewModel.preeditText)")
        Text("Buffer: \(viewModel.bufferText)")
        
        Text("Candidates:")
        ScrollView {
          ForEach(viewModel.candidates.indices, id: \.self) { idx in
            Button(action: {
              DispatchQueue.global(qos: .userInitiated).async {
                viewModel.selectCandidate(at: idx)
              }
            }) {
              Text("\(idx + 1). \(viewModel.candidates[idx])")
            }
          }
        }
      }
      .padding()
      .background(Color(UIColor.secondarySystemBackground))
      .cornerRadius(8)
      Spacer()
      // Input field to type characters
      HStack {
        TextField("Type character", text: $userInput)
          .textFieldStyle(RoundedBorderTextFieldStyle())
          .frame(width: 100)
          .keyboardType(.asciiCapable)
          .textInputAutocapitalization(.never)
          .onChange(of: userInput) { newValue in
            if newValue.isEmpty { return }

            DispatchQueue.global(qos: .userInteractive).async {
              newValue.forEach {
                viewModel.process(character: $0)
              }
            }
            userInput = ""
          }
        
        Button("Enter") {
          DispatchQueue.global(qos: .userInitiated).async {
            viewModel.process(character: .enter)
          }
        }
        Button("Space") {
          DispatchQueue.global(qos: .userInitiated).async {
            viewModel.process(character: .space)
          }
        }
        Button("Backspace") {
          DispatchQueue.global(qos: .userInitiated).async {
            viewModel.process(character: .backspace)
          }
        }
      }
      
      Spacer()
    }
    .padding()
  }
}

#Preview {
  ContentView()
}
