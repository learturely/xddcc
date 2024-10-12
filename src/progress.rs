pub trait ProgressTrackerHolder<P: ProgressTracker> {
    fn init(&self, total: u64, data: &str) -> P;
    fn remove_progress(&self, progress: &P);
}
pub trait ProgressTracker: Send + Sized {
    fn inc(&self, delta: u64);
    fn finish(&self, progress_bar_holder: &impl ProgressTrackerHolder<Self>, msg: &'static str);
}