#[derive(Debug, Clone)]
pub enum Event {
    TaskStarted(String),
    TaskFinished(String),
}
