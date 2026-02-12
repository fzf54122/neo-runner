use runner_core::domain::{JobSpec, RetrySpec, TaskSpec};
use serde::Deserialize;
use std::collections::HashSet;
use std::fs;

const CONFIG_VERSION: u32 = 1;

#[derive(Debug, Deserialize)]
struct RawConfig {
    job: RawJob,
    version: u32,
}

#[derive(Debug, Deserialize)]
struct RawJob {
    name: String,
    tasks: Vec<RawTask>,
    fail_fast: Option<bool>,
    max_concurrency: Option<usize>,
    default_timeout: Option<String>,
    default_retry: Option<RawRetry>,
}

#[derive(Debug, Deserialize)]
struct RawTask {
    id: String,
    // yaml里面叫type，但是rust不允许用type作为变量名，所以用task_type
    #[serde(rename = "type")]
    task_type: String,
    // 这样yaml 不写depends_on 也不会报错
    #[serde(default)]
    depends_on: Vec<String>,
    cmd: Option<String>,
    method: Option<String>,
    url: Option<String>,
    expected_status: Option<RawExpectedStatus>,
    timeout: Option<String>,
    retry: Option<RawRetry>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum RawExpectedStatus {
    One(u16),
    Many(Vec<u16>),
}

#[derive(Debug, Deserialize)]
struct RawRetry {
    max_attempts: Option<u32>,
}

pub fn load_yaml(path: &str) -> Result<JobSpec, String> {
    let content = fs::read_to_string(path)
        // 把底层错误转换函数要求的string
        .map_err(|e| format!("read file failed: {}", e))?;
    load_yaml_str(&content)
}

// 纯解析/校验层
pub fn load_yaml_str(content: &str) -> Result<JobSpec, String> {
    let raw: RawConfig =
        serde_yaml::from_str(content).map_err(|e| format!("parse yaml failed: {}", e))?;

    let fail_fast: bool = raw.job.fail_fast.unwrap_or(true);
    let max_concurrency = raw.job.max_concurrency.unwrap_or(1);
    let default_timeout_ms = raw
        .job
        .default_timeout
        .as_deref()
        .map(parse_duration_ms)
        .transpose()?;
    let default_max_attempts = raw
        .job
        .default_retry
        .as_ref() // 不拿走所有权，只是引用
        .and_then(|r| r.max_attempts)
        .unwrap_or(1);
    if default_max_attempts == 0 {
        return Err("default max attempts not allowed".to_string());
    }
    let default_retry = RetrySpec {
        max_attempts: default_max_attempts,
    };

    if raw.version != CONFIG_VERSION {
        return Err(format!(
            "unsupported version: got {}, expected {}",
            raw.version, CONFIG_VERSION
        ));
    }

    if max_concurrency == 0 {
        return Err("max concurrency not allowed".to_string());
    }

    if raw.job.name.trim().is_empty() {
        return Err("job.name cannot be empty".to_string());
    }

    if raw.job.tasks.is_empty() {
        return Err("job.tasks cannot be empty".to_string());
    }

    // 检查task id是否重复
    let mut seen = HashSet::new();

    let tasks = raw
        .job
        .tasks
        .into_iter()
        .map(|raw_task| {
            let depends_on = raw_task
                .depends_on
                .into_iter()
                .map(|d| d.trim().to_string())
                .filter(|d| !d.is_empty())
                .collect::<Vec<_>>();
            let id = raw_task.id.trim();
            let task_type = raw_task.task_type.trim();
            let cmd = raw_task
                .cmd
                .as_deref()
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map(ToString::to_string);
            let method = raw_task
                .method
                .as_deref()
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map(ToString::to_string);
            let url = raw_task
                .url
                .as_deref()
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map(ToString::to_string);
            let expected_status = match raw_task.expected_status {
                Some(RawExpectedStatus::One(v)) => Some(vec![v]),
                Some(RawExpectedStatus::Many(vs)) => Some(vs),
                None => None,
            };

            if id.is_empty() {
                return Err("task.id cannot be empty".to_string());
            }
            // insert 返回true/false 如果是false说明已经存在了
            if !seen.insert(id.to_string()) {
                return Err(format!("duplicate task id: '{}'", id));
            }

            if task_type.is_empty() {
                return Err("task.type cannot be empty".to_string());
            }
            // 任务超时时间可以为空，为空时用默认值
            let task_timeout_ms = raw_task
                .timeout
                .as_deref()
                .map(parse_duration_ms)
                .transpose()?;
            // 如果任务超时时间为空，用默认值
            let timeout_ms = task_timeout_ms.or(default_timeout_ms);
            let retry = match raw_task.retry.as_ref().and_then(|r| r.max_attempts) {
                Some(v) => {
                    if v == 0 {
                        return Err("task retry max attempts not allowed".to_string());
                    }
                    Some(RetrySpec { max_attempts: v })
                }
                None => None,
            };

            Ok(TaskSpec {
                id: id.to_string(),
                task_type: task_type.to_string(), // 存trim后的值
                cmd,
                method,
                url,
                expected_status,
                depends_on,
                timeout_ms,
                retry,
            })
        })
        // 收集所有结果，有一个失败就返回失败，自动返回失败
        .collect::<Result<Vec<_>, _>>()?;

    let all_ids: HashSet<String> = tasks.iter().map(|t| t.id.clone()).collect();

    for task in &tasks {
        for dep in &task.depends_on {
            // 不能依赖自己
            if dep == &task.id {
                return Err(format!("task '{}' depends on itself", task.id));
            }
            // 依赖必须存在
            if !all_ids.contains(dep) {
                return Err(format!(
                    "task '{}' depends on non-existent task '{}'",
                    task.id, dep
                ));
            }
        }
    }

    Ok(JobSpec {
        name: raw.job.name,
        fail_fast,
        max_concurrency,
        tasks,
        default_timeout_ms,
        default_retry,
    })
}

fn parse_duration_ms(raw: &str) -> Result<u64, String> {
    let s = raw.trim();

    if s.is_empty() {
        return Err("duration cannot be empty".to_string());
    }
    //
    if let Some(num) = s.strip_suffix("ms") {
        return num
            .trim()
            .parse::<u64>()
            .map_err(|_| format!("invalid duration '{}'", raw));
    }
    if let Some(num) = s.strip_suffix('s') {
        let v = num
            .trim()
            .parse::<u64>()
            .map_err(|_| format!("invalid duration '{}'", raw))?;
        return v
            .checked_mul(1_000)
            .ok_or_else(|| format!("duration overflow '{}'", raw));
    }
    if let Some(num) = s.strip_suffix('m') {
        let v = num
            .trim()
            .parse::<u64>()
            .map_err(|_| format!("invalid duration '{}'", raw))?;
        return v
            .checked_mul(60_000)
            .ok_or_else(|| format!("duration overflow '{}'", raw));
    }
    if let Some(num) = s.strip_suffix('h') {
        let v = num
            .trim()
            .parse::<u64>()
            .map_err(|_| format!("invalid duration '{}'", raw))?;
        return v
            .checked_mul(3_600_000)
            .ok_or_else(|| format!("duration overflow '{}'", raw));
    }
    Err(format!("invalid duration '{}': expected ms/s/m/h", raw))
}

#[cfg(test)]
mod tests {
    use super::*;

    // 成功解析
    #[test]
    fn load_yaml_str_ok() {
        let yaml = r#"
            version: 1
            job:
                name: demo
                tasks:
                    - id: task1
                      type: shell
            "#;

        let result = load_yaml_str(yaml);
        assert!(result.is_ok());
    }

    // 失败解析：重复task id
    #[test]
    fn load_yaml_str_duplicates_task_id_err() {
        let yaml = r#"
            version: 1
            job:
                name: demo
                tasks:
                    - id: task1
                      type: shell
                    - id: task1
                      type: shell
        "#;
        let err = load_yaml_str(yaml).unwrap_err();
        assert!(err.contains("duplicate task id"));
    }

    // 失败解析：不支持的版本
    #[test]
    fn load_yaml_str_version_err() {
        let yaml = r#"
            version: 2
            job:
                name: demo
                tasks:
                    - id: task1
                      type: shell
        "#;
        let err = load_yaml_str(yaml).unwrap_err();
        assert!(err.contains("unsupported version"));
    }

    // 失败解析：未知依赖
    #[test]
    fn load_yaml_str_unknown_dependency() {
        let yaml = r#"
            version: 1
            job:
                name: demo
                tasks:
                    - id: task1
                      type: shell
                    - id: task2
                      type: shell
                      depends_on:
                        - task3
        "#;
        let err = load_yaml_str(yaml).unwrap_err();
        assert!(err.contains("task 'task2' depends on non-existent task 'task3'"));
    }

    // 失败解析：循环依赖
    #[test]
    fn load_yaml_str_self_dependency_err() {
        let yaml = r#"
        version: 1
        job:
          name: demo
          tasks:
            - id: task1
              type: shell
              depends_on: [task1]
        "#;
        let err = load_yaml_str(yaml).unwrap_err();
        assert!(err.contains("depends on itself"));
    }

    #[test]
    fn load_yaml_str_default_timeout_applies_to_task() {
        let yaml = r#"
version: 1
job:
  name: demo
  default_timeout: 30s
  tasks:
    - id: task1
      type: shell
"#;

        let job = load_yaml_str(yaml).expect("should parse");
        assert_eq!(job.default_timeout_ms, Some(30_000));
        assert_eq!(job.tasks.len(), 1);
        assert_eq!(job.tasks[0].timeout_ms, Some(30_000));
    }

    #[test]
    fn load_yaml_str_task_timeout_overrides_default() {
        let yaml = r#"
version: 1
job:
  name: demo
  default_timeout: 30s
  tasks:
    - id: task1
      type: shell
      timeout: 5s
"#;

        let job = load_yaml_str(yaml).expect("should parse");
        assert_eq!(job.default_timeout_ms, Some(30_000));
        assert_eq!(job.tasks.len(), 1);
        assert_eq!(job.tasks[0].timeout_ms, Some(5_000));
    }

    #[test]
    fn load_yaml_str_invalid_duration_err() {
        let yaml = r#"
version: 1
job:
  name: demo
  default_timeout: 10x
  tasks:
    - id: task1
      type: shell
"#;

        let err = load_yaml_str(yaml).unwrap_err();
        assert!(err.contains("invalid duration"));
    }

    #[test]
    fn load_yaml_str_empty_duration_err() {
        let yaml = r#"
version: 1
job:
  name: demo
  default_timeout: "   "
  tasks:
    - id: task1
      type: shell
"#;

        let err = load_yaml_str(yaml).unwrap_err();
        assert!(err.contains("empty") || err.contains("cannot be empty"));
    }

    #[test]
    fn load_yaml_str_default_retry_defaults_to_one() {
        let yaml = r#"
version: 1
job:
  name: demo
  tasks:
    - id: task1
      type: shell
"#;

        let job = load_yaml_str(yaml).expect("should parse");
        assert_eq!(job.default_retry.max_attempts, 1);
    }

    #[test]
    fn load_yaml_str_job_default_retry_applies() {
        let yaml = r#"
version: 1
job:
  name: demo
  default_retry:
    max_attempts: 3
  tasks:
    - id: task1
      type: shell
"#;

        let job = load_yaml_str(yaml).expect("should parse");
        assert_eq!(job.default_retry.max_attempts, 3);
    }

    #[test]
    fn load_yaml_str_task_retry_parsed() {
        let yaml = r#"
version: 1
job:
  name: demo
  default_retry:
    max_attempts: 3
  tasks:
    - id: task1
      type: shell
      retry:
        max_attempts: 2
"#;

        let job = load_yaml_str(yaml).expect("should parse");
        assert_eq!(job.default_retry.max_attempts, 3);
        assert_eq!(job.tasks.len(), 1);
        assert_eq!(job.tasks[0].retry.as_ref().map(|r| r.max_attempts), Some(2));
    }

    #[test]
    fn load_yaml_str_retry_zero_err() {
        let yaml_job_retry_zero = r#"
version: 1
job:
  name: demo
  default_retry:
    max_attempts: 0
  tasks:
    - id: task1
      type: shell
"#;
        let err = load_yaml_str(yaml_job_retry_zero).unwrap_err();
        assert!(err.contains("default max attempts"));

        let yaml_task_retry_zero = r#"
version: 1
job:
  name: demo
  tasks:
    - id: task1
      type: shell
      retry:
        max_attempts: 0
"#;
        let err = load_yaml_str(yaml_task_retry_zero).unwrap_err();
        assert!(err.contains("task retry max attempts"));
    }

    #[test]
    fn load_yaml_str_http_fields_parsed() {
        let yaml = r#"
version: 1
job:
  name: demo
  tasks:
    - id: health
      type: http
      method: GET
      url: https://example.com/health
      expected_status: [200, 204]
"#;

        let job = load_yaml_str(yaml).expect("should parse http fields");
        assert_eq!(job.tasks.len(), 1);
        let t = &job.tasks[0];
        assert_eq!(t.task_type, "http");
        assert_eq!(t.method.as_deref(), Some("GET"));
        assert_eq!(t.url.as_deref(), Some("https://example.com/health"));
        assert_eq!(t.expected_status.as_ref().map(|v| v.len()), Some(2));
    }
}
