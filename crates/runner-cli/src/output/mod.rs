pub fn print_result(success: bool, total: usize) {
    println!("success={success} total={total}");
}

pub fn print_plan(task_ids: &[String]) {
    for id in task_ids {
        println!("{id}");
    }
}

pub fn print_validate_ok(task_total: usize) {
    println!("config valid, tasks={task_total}");
}
