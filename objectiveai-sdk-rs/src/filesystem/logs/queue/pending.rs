/// Handle to a notification whose log file has been written and
/// whose per-agent DB index has been reserved by
/// [`super::handle::Queue::write_notification`].
///
/// The cli-stream writer task queues these locally and passes them
/// back into [`super::handle::Queue::insert_notification`] when the
/// next tool response for the same agent comes in — or at stream
/// end via the writer's `finalize`.
#[derive(Debug, Clone)]
pub struct PendingNotification {
    pub agent_id: String,
    pub index: u64,
    pub path: String,
    pub timestamp: u64,
}
