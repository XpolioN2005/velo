use velo_exec::executor::system::DefaultSystem;
use velo_exec::*;

// ── dummy system ─────────────────────────

struct DummySystem;

impl SystemHandler for DummySystem {
    fn run(&self, _: SystemActionId, _: &mut Context) -> StepResult {
        StepResult {
            success: true,
            value: Value::None,
            error: None,
        }
    }
}

// ── helpers ─────────────────────────────

fn executor() -> Executor<DummySystem> {
    Executor::new(DummySystem)
}

// ── tests ───────────────────────────────

#[test]
fn basic_execution_updates_last() {
    let exec = executor();

    let steps = vec![Step::Action {
        action: Action::Transform(Transform::Regex {
            input: "abc123".into(),
            pattern: r"(\d+)".into(),
            group: 1,
        }),
        assign_to: None,
    }];

    let mut ctx = Context::new(vec![]);

    let result = exec.run(&steps, &mut ctx);

    assert!(result.success);

    match ctx.last {
        Value::String(ref s) => assert_eq!(s, "123"),
        _ => panic!("Expected string"),
    }
}

#[test]
fn variable_assignment_works() {
    let exec = executor();

    let steps = vec![Step::Action {
        action: Action::Transform(Transform::Regex {
            input: "val=42".into(),
            pattern: r"=(\d+)".into(),
            group: 1,
        }),
        assign_to: Some("num".into()),
    }];

    let mut ctx = Context::new(vec![]);

    exec.run(&steps, &mut ctx);

    match ctx.vars.get("num") {
        Some(Value::String(s)) => assert_eq!(s, "42"),
        _ => panic!("Variable not assigned correctly"),
    }
}

#[test]
fn placeholder_args_work() {
    let exec = executor();

    let steps = vec![Step::Action {
        action: Action::Transform(Transform::Regex {
            input: "{0}".into(),
            pattern: r"(hello)".into(),
            group: 1,
        }),
        assign_to: None,
    }];

    let mut ctx = Context::new(vec!["hello".into()]);

    let result = exec.run(&steps, &mut ctx);

    assert!(result.success);
}

#[test]
fn placeholder_vars_work() {
    let exec = executor();

    let mut ctx = Context::new(vec![]);
    ctx.vars
        .insert("name".into(), Value::String("world".into()));

    let steps = vec![Step::Action {
        action: Action::Transform(Transform::Regex {
            input: "hello {var:name}".into(),
            pattern: r"hello (.+)".into(),
            group: 1,
        }),
        assign_to: None,
    }];

    let result = exec.run(&steps, &mut ctx);

    match result.value {
        Value::String(s) => assert_eq!(s, "world"),
        _ => panic!("Expected string"),
    }
}

#[test]
fn process_capture_works() {
    let exec = executor();

    let steps = vec![Step::Action {
        action: Action::LaunchProcess {
            program: "cmd".into(),
            args: vec!["/C".into(), "echo hello".into()],
            mode: ExecMode::Capture,
        },
        assign_to: None,
    }];

    let mut ctx = Context::new(vec![]);

    let result = exec.run(&steps, &mut ctx);

    assert!(result.success);

    match result.value {
        Value::String(s) => {
            assert!(s.to_lowercase().contains("hello"));
        }
        _ => panic!("Expected string output"),
    }
}

#[test]
fn transform_regex_works() {
    let exec = executor();

    let steps = vec![Step::Action {
        action: Action::Transform(Transform::Regex {
            input: "abc999xyz".into(),
            pattern: r"(\d+)".into(),
            group: 1,
        }),
        assign_to: None,
    }];

    let mut ctx = Context::new(vec![]);

    let result = exec.run(&steps, &mut ctx);

    match result.value {
        Value::String(s) => assert_eq!(s, "999"),
        _ => panic!("Expected string"),
    }
}

#[test]
fn failure_stops_execution() {
    let exec = executor();

    let steps = vec![
        Step::Action {
            action: Action::Transform(Transform::Regex {
                input: "abc".into(),
                pattern: r"(\d+)".into(), // will fail
                group: 1,
            }),
            assign_to: Some("x".into()),
        },
        Step::Action {
            action: Action::Transform(Transform::Regex {
                input: "123".into(),
                pattern: r"(\d+)".into(),
                group: 1,
            }),
            assign_to: Some("y".into()),
        },
    ];

    let mut ctx = Context::new(vec![]);

    let result = exec.run(&steps, &mut ctx);

    assert!(!result.success);
    assert!(ctx.vars.get("y").is_none()); // second step must NOT run
}

#[test]
fn system_get_and_set_cwd() {
    let exec = Executor::new(DefaultSystem);

    let steps = vec![
        Step::Action {
            action: Action::System(SystemActionId::GetCwd),
            assign_to: Some("cwd".into()),
        },
        Step::Action {
            action: Action::System(SystemActionId::SetCwd),
            assign_to: None,
        },
    ];

    let mut ctx = Context::new(vec![]);

    let result = exec.run(&steps, &mut ctx);

    assert!(result.success);
    assert!(ctx.cwd.is_some());
}
