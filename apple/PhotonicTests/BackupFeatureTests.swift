import ComposableArchitecture
import Foundation
import Testing
@testable import Photonic

@MainActor
struct BackupFeatureTests {
    @Test
    func onAppearRecoversStuckJobsAndRehydrates() async {
        let store = TestStore(initialState: BackupFeature.State()) {
            BackupFeature()
        } withDependencies: {
            $0.backupQueueClient.recoverStuckUploads = {}
            $0.backupQueueClient.observe = {
                AsyncStream { continuation in
                    continuation.yield(BackupQueueSnapshot(pending: 2, done: 1))
                    continuation.finish()
                }
            }
        }

        await store.send(.onAppear)
        await store.receive(.queueDidChange(BackupQueueSnapshot(pending: 2, done: 1))) {
            $0.snapshot = BackupQueueSnapshot(pending: 2, done: 1)
        }
    }

    @Test
    func startProcessesPendingJobs() async {
        let job1 = UploadJob(albumID: "a", albumName: "Album", mediaID: "m1")
        let job2 = UploadJob(albumID: "a", albumName: "Album", mediaID: "m2")
        let queue = FakeQueue(jobs: [job1, job2])

        let store = TestStore(
            initialState: BackupFeature.State(snapshot: BackupQueueSnapshot(pending: 2))
        ) {
            BackupFeature()
        } withDependencies: {
            let (queueClient, uploadClient) = makeBackupDeps(queue: queue)
            $0.backupQueueClient = queueClient
            $0.uploadClient = uploadClient
        }

        await store.send(.startTapped) {
            $0.phase = .processing
        }
        await store.receive(.jobDidStart(job1)) {
            $0.currentJob = job1
            $0.snapshot.pending = 1
            $0.snapshot.uploading = 1
        }
        await store.receive(.jobDidFinish(job1.id, .success)) {
            $0.currentJob = nil
            $0.snapshot.uploading = 0
            $0.snapshot.done = 1
        }
        await store.receive(.jobDidStart(job2)) {
            $0.currentJob = job2
            $0.snapshot.pending = 0
            $0.snapshot.uploading = 1
        }
        await store.receive(.jobDidFinish(job2.id, .success)) {
            $0.currentJob = nil
            $0.snapshot.uploading = 0
            $0.snapshot.done = 2
        }
        await store.receive(.processingFinished) {
            $0.phase = .completed
        }
    }

    @Test
    func failedUploadMarksJobFailedAndContinues() async {
        let job1 = UploadJob(albumID: "a", albumName: "Album", mediaID: "m1")
        let job2 = UploadJob(albumID: "a", albumName: "Album", mediaID: "m2")
        let queue = FakeQueue(jobs: [job1, job2])

        let store = TestStore(
            initialState: BackupFeature.State(snapshot: BackupQueueSnapshot(pending: 2))
        ) {
            BackupFeature()
        } withDependencies: {
            let (queueClient, uploadClient) = makeBackupDeps(queue: queue) { job in
                if job.mediaID == "m1" {
                    throw TestUploadFailure()
                }
            }
            $0.backupQueueClient = queueClient
            $0.uploadClient = uploadClient
        }

        await store.send(.startTapped) {
            $0.phase = .processing
        }
        await store.receive(.jobDidStart(job1)) {
            $0.currentJob = job1
            $0.snapshot.pending = 1
            $0.snapshot.uploading = 1
        }
        await store.receive(.jobDidFinish(job1.id, .failure("upload failed"))) {
            $0.currentJob = nil
            $0.snapshot.uploading = 0
            $0.snapshot.failed = 1
            $0.lastErrorMessage = "upload failed"
        }
        await store.receive(.jobDidStart(job2)) {
            $0.currentJob = job2
            $0.snapshot.pending = 0
            $0.snapshot.uploading = 1
        }
        await store.receive(.jobDidFinish(job2.id, .success)) {
            $0.currentJob = nil
            $0.snapshot.uploading = 0
            $0.snapshot.done = 1
        }
        await store.receive(.processingFinished) {
            $0.phase = .completed
        }
    }

    @Test
    func pauseDuringUploadReturnsJobToPendingAndResumeContinues() async {
        let job = UploadJob(albumID: "a", albumName: "Album", mediaID: "m1")
        let queue = FakeQueue(jobs: [job])
        let uploader = FlakyUploader()

        let store = TestStore(
            initialState: BackupFeature.State(snapshot: BackupQueueSnapshot(pending: 1))
        ) {
            BackupFeature()
        } withDependencies: {
            let (queueClient, uploadClient) = makeBackupDeps(queue: queue) { job in
                try await uploader.handle(job)
            }
            $0.backupQueueClient = queueClient
            $0.uploadClient = uploadClient
        }

        await store.send(.startTapped) {
            $0.phase = .processing
        }
        await store.receive(.jobDidStart(job)) {
            $0.currentJob = job
            $0.snapshot.pending = 0
            $0.snapshot.uploading = 1
        }
        await store.send(.pauseTapped) {
            $0.phase = .paused
            $0.currentJob = nil
            $0.snapshot.uploading = 0
            $0.snapshot.pending = 1
        }
        await store.send(.resumeTapped) {
            $0.phase = .processing
        }
        await store.receive(.jobDidStart(job)) {
            $0.currentJob = job
            $0.snapshot.pending = 0
            $0.snapshot.uploading = 1
        }
        await store.receive(.jobDidFinish(job.id, .success)) {
            $0.currentJob = nil
            $0.snapshot.uploading = 0
            $0.snapshot.done = 1
        }
        await store.receive(.processingFinished) {
            $0.phase = .completed
        }
    }

    @Test
    func cancelReturnsToIdleWithJobPending() async {
        let job = UploadJob(albumID: "a", albumName: "Album", mediaID: "m1")
        let queue = FakeQueue(jobs: [job])
        let uploader = FlakyUploader()

        let store = TestStore(
            initialState: BackupFeature.State(snapshot: BackupQueueSnapshot(pending: 1))
        ) {
            BackupFeature()
        } withDependencies: {
            let (queueClient, uploadClient) = makeBackupDeps(queue: queue) { job in
                try await uploader.handle(job)
            }
            $0.backupQueueClient = queueClient
            $0.uploadClient = uploadClient
        }

        await store.send(.startTapped) {
            $0.phase = .processing
        }
        await store.receive(.jobDidStart(job)) {
            $0.currentJob = job
            $0.snapshot.pending = 0
            $0.snapshot.uploading = 1
        }
        await store.send(.cancelTapped) {
            $0.phase = .idle
            $0.currentJob = nil
            $0.snapshot.uploading = 0
            $0.snapshot.pending = 1
        }
    }

    @Test
    func selectionEnqueuesJobsAndStartsProcessing() async {
        let album = PhotoAlbum(id: "album-1", name: "Album", assetCount: 2)
        let pending = [
            PendingMedia(id: "m1", type: .photo, filename: "IMG_1.HEIC", dateTaken: nil),
            PendingMedia(id: "m2", type: .video, filename: "VID_1.MOV", dateTaken: nil)
        ]
        let queue = FakeQueue(jobs: [])

        let store = TestStore(initialState: BackupFeature.State()) {
            BackupFeature()
        } withDependencies: {
            let (queueClient, uploadClient) = makeBackupDeps(queue: queue)
            $0.backupQueueClient = queueClient
            $0.uploadClient = uploadClient
            $0.photoLibraryClient = FakePhotos(albums: [album], pending: pending).makeClient()
        }

        await store.send(.chooseAlbumsTapped) {
            $0.isLoadingAlbums = true
        }
        await store.receive(.albumsLoaded([album])) {
            $0.isLoadingAlbums = false
            $0.albums = [album]
        }
        await store.send(.albumToggled("album-1")) {
            $0.selectedAlbumIDs = ["album-1"]
        }
        await store.send(.enqueueSelectionTapped) {
            $0.isPreparingSelection = true
        }
        await store.receive(\.selectionQueued) {
            $0.isPreparingSelection = false
            $0.snapshot.pending = 2
            $0.selectedAlbumIDs = []
        }
        #expect(queue.enqueuedMediaIDs() == ["m1", "m2"])
        let queuedJobs = queue.enqueuedJobs()

        await store.receive(\.startTapped) {
            $0.phase = .processing
        }
        await store.receive(\.jobDidStart) {
            $0.currentJob = queuedJobs[0]
            $0.snapshot.pending = 1
            $0.snapshot.uploading = 1
        }
        await store.receive(\.jobDidFinish) {
            $0.currentJob = nil
            $0.snapshot.uploading = 0
            $0.snapshot.done = 1
        }
        await store.receive(\.jobDidStart) {
            $0.currentJob = queuedJobs[1]
            $0.snapshot.pending = 0
            $0.snapshot.uploading = 1
        }
        await store.receive(\.jobDidFinish) {
            $0.currentJob = nil
            $0.snapshot.uploading = 0
            $0.snapshot.done = 2
        }
        await store.receive(\.processingFinished) {
            $0.phase = .completed
        }
    }
}
