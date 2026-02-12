pub trait TaskExecutor {
    fn execute(&self, task_id: &str) -> Result<(), String>;
}
