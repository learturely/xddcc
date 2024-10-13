pub trait ProgressTrackerHolder<P: ProgressTracker> {
    fn init(&self, total: u64, data: ProgressState) -> P;
    fn remove_progress(&self, progress: &P);
}
pub trait ProgressTracker: Send + Sized {
    fn inc(&self, delta: u64);
    fn go_on(&self) -> bool {
        true
    }
    fn finish(&self, data: ProgressState);
}
#[non_exhaustive]
pub enum ProgressState {
    GetRecordingLives,
    GetLiveIds,
    GetLiveUrls,
    GetDeviceCodes,
}
