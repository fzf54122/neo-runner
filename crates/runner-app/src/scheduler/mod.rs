use std::collections::{HashMap, VecDeque};
use runner_core::domain::{JobSpec, TaskSpec};

pub fn build_plan<'a>(job: &'a JobSpec) -> Result<Vec<&'a TaskSpec>, String> {
    // id -> task 引用
    let mut task_map: HashMap<&str, &TaskSpec> = HashMap::new();
    // 每个任务的入度（有多少前置依赖）
    let mut indegree: HashMap<&str, usize> = HashMap::new();
    // dep -> [依赖 dep 的任务id列表]
    let mut graph: HashMap<&str, Vec<&str>> = HashMap::new();
    // 初始化
    for task in &job.tasks {
        let id = task.id.as_str();
        task_map.insert(id, task);
        indegree.insert(id, 0);
        graph.entry(id).or_default();
    }
    // 建图 + 计算入度
    for task in &job.tasks {
        let task_id = task.id.as_str();
        for dep in &task.depends_on {
            let dep_id = dep.as_str();
            if !task_map.contains_key(dep_id) {
                return Err(format!(
                    "task '{}' depends on unknown task '{}'",
                    task_id, dep_id
                ));
            }
            graph.entry(dep_id).or_default().push(task_id);
            if let Some(v) = indegree.get_mut(task_id) {
                *v += 1;
            }
        }
    }
    // 入度为 0 的先入队
    let mut queue = VecDeque::new();
    for (id, deg) in &indegree {
        if *deg == 0 {
            queue.push_back(*id);
        }
    }
    // 拓扑排序
    let mut plan: Vec<&TaskSpec> = Vec::new();
    while let Some(id) = queue.pop_front() {
        let task = task_map
            .get(id)
            .ok_or_else(|| format!("internal error: missing task '{}'", id))?;
        plan.push(*task);
        if let Some(next_ids) = graph.get(id) {
            for next_id in next_ids {
                if let Some(v) = indegree.get_mut(next_id) {
                    *v -= 1;
                    if *v == 0 {
                        queue.push_back(*next_id);
                    }
                }
            }
        }
    }
    if plan.len() != job.tasks.len() {
        return Err("dependency cycle detected".to_string());
    }
    Ok(plan)
}