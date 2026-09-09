import ComposableArchitecture
import PhotonicCore
import SwiftUI

struct LibraryView: View {
    let store: StoreOf<LibraryFeature>

    var body: some View {
        Group {
            if store.media.isEmpty, !store.isLoading {
                ContentUnavailableView {
                    Label("Library", systemImage: "photo.on.rectangle")
                } description: {
                    Text(store.errorMessage ?? "Nothing backed up yet.")
                }
            } else {
                List {
                    ForEach(store.media) { medium in
                        row(medium)
                    }
                    footer
                }
            }
        }
        .onAppear {
            guard !AppRuntime.isRunningUnitTests else { return }
            store.send(.onAppear)
        }
    }

    private func row(_ medium: Medium) -> some View {
        HStack(spacing: 12) {
            Image(systemName: icon(for: medium.type))
                .foregroundStyle(.secondary)
                .frame(width: 28)
            VStack(alignment: .leading, spacing: 2) {
                Text(medium.primaryFilename ?? medium.id.uuidString)
                    .font(.subheadline)
                    .lineLimit(1)
                HStack(spacing: 6) {
                    Text(medium.type.rawValue.lowercased())
                        .font(.caption2)
                        .foregroundStyle(.secondary)
                    if let date = medium.takenAt {
                        Text(date, style: .date)
                            .font(.caption2)
                            .foregroundStyle(.secondary)
                    }
                    if let size = medium.primaryFilesize {
                        Text(ByteCountFormatter.string(fromByteCount: size, countStyle: .file))
                            .font(.caption2)
                            .foregroundStyle(.secondary)
                    }
                }
            }
        }
        .padding(.vertical, 2)
        .onAppear {
            if medium.id == store.media.suffix(5).first?.id {
                store.send(.loadNextPage)
            }
        }
    }

    @ViewBuilder
    private var footer: some View {
        if store.isLoading {
            HStack {
                Spacer()
                ProgressView()
                Spacer()
            }
        } else if store.hasMore {
            Button("Load more") {
                store.send(.loadNextPage)
            }
            .frame(maxWidth: .infinity)
        }
    }

    private func icon(for type: MediumType) -> String {
        switch type {
        case .photo: "photo"
        case .video: "video"
        case .livePhoto: "livephoto"
        case .vector: "square.and.line.vertical.and.square"
        case .sequence, .gif: "square.stack.3d.up"
        case .other: "doc"
        }
    }
}

#Preview {
    LibraryView(
        store: Store(initialState: LibraryFeature.State()) {
            LibraryFeature()
        } withDependencies: {
            $0.mediaClient.fetchPage = { _, pageSize in
                MediaPage(
                    media: (0 ..< pageSize).map { index in
                        Medium(
                            id: UUID(),
                            type: index.isMultiple(of: 2) ? .photo : .video,
                            albumID: nil,
                            takenAt: Date(timeIntervalSinceNow: -Double(index) * 3600),
                            primaryFilename: "IMG_00\(index).HEIC",
                            primaryFilesize: 2_400_000
                        )
                    },
                    nextCursor: nil
                )
            }
        }
    )
}
