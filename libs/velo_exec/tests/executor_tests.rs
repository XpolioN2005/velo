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
            input: Some("abc123".into()),
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
            input: Some("val=42".into()),
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
            input: Some("{0}".into()),
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
            input: Some("hello {var:name}".into()),
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
fn placeholder_last_works() {
    let exec = executor();

    let steps = vec![
        Step::Action {
            action: Action::Transform(Transform::Regex {
                input: Some("abc123".into()),
                pattern: r"(\d+)".into(),
                group: 1,
            }),
            assign_to: None,
        },
        Step::Action {
            action: Action::Transform(Transform::Regex {
                input: Some("{last}".into()),
                pattern: r"(123)".into(),
                group: 1,
            }),
            assign_to: Some("num".into()),
        },
    ];

    let mut ctx = Context::new(vec![]);

    let result = exec.run(&steps, &mut ctx);

    assert!(result.success);

    match ctx.vars.get("num") {
        Some(Value::String(s)) => assert_eq!(s, "123"),
        _ => panic!("{{last}} not working"),
    }
}

#[test]
fn process_capture_works() {
    let exec = executor();

    let steps = vec![Step::Action {
        action: Action::LaunchProcess {
            program: "echo".into(),
            args: vec!["hello".into()],
            mode: ExecMode::Capture,
            shell: true,
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
            input: Some("abc999xyz".into()),
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
fn transform_uses_ctx_last_when_none() {
    let exec = executor();

    let steps = vec![
        Step::Action {
            action: Action::Transform(Transform::Regex {
                input: Some("file.rs:123".into()),
                pattern: r"(.+):\d+".into(),
                group: 1,
            }),
            assign_to: None,
        },
        Step::Action {
            action: Action::Transform(Transform::Regex {
                input: None,
                pattern: r"(.+)\.rs".into(),
                group: 1,
            }),
            assign_to: Some("name".into()),
        },
    ];

    let mut ctx = Context::new(vec![]);

    let result = exec.run(&steps, &mut ctx);

    assert!(result.success);

    match ctx.vars.get("name") {
        Some(Value::String(s)) => assert_eq!(s, "file"),
        _ => panic!("ctx.last chaining failed"),
    }
}

#[test]
fn real_rg_pipeline() {
    let exec = executor();

    let steps = vec![
        Step::Action {
            action: Action::LaunchProcess {
                program: "rg".into(),
                args: vec!["fn".into(), ".".into(), "--max-count".into(), "100".into()],
                mode: ExecMode::Capture,
                shell: false,
            },
            assign_to: None,
        },
        Step::Action {
            action: Action::Transform(Transform::Regex {
                input: None,
                pattern: r"^([^:\n]+)".into(),
                group: 1,
            }),
            assign_to: Some("file".into()),
        },
        Step::Action {
            action: Action::LaunchProcess {
                program: "code".into(),
                args: vec!["{var:file}".into()],
                mode: ExecMode::FireForget,
                shell: true,
            },
            assign_to: None,
        },
    ];

    let mut ctx = Context::new(vec![]);
    ctx.cwd = Some(std::env::current_dir().unwrap());

    let result = exec.run(&steps, &mut ctx);

    println!("RAW OUTPUT = {:?}", ctx.last);
    println!("file = {:?}", ctx.vars.get("file"));

    assert!(result.success);
}

#[test]
fn failure_stops_execution() {
    let exec = executor();

    let steps = vec![
        Step::Action {
            action: Action::Transform(Transform::Regex {
                input: Some("abc".into()),
                pattern: r"(\d+)".into(),
                group: 1,
            }),
            assign_to: Some("x".into()),
        },
        Step::Action {
            action: Action::Transform(Transform::Regex {
                input: Some("123".into()),
                pattern: r"(\d+)".into(),
                group: 1,
            }),
            assign_to: Some("y".into()),
        },
    ];

    let mut ctx = Context::new(vec![]);

    let result = exec.run(&steps, &mut ctx);

    assert!(!result.success);
    assert!(ctx.vars.get("y").is_none());
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

#[test]
fn split_and_first_pipeline() {
    let exec = executor();

    let steps = vec![
        // simulate rg output
        Step::Action {
            action: Action::Transform(Transform::Regex {
                input: Some("src/main.rs:10\nsrc/lib.rs:20".into()),
                pattern: r"(.+)".into(),
                group: 1,
            }),
            assign_to: None,
        },
        // split lines
        Step::Action {
            action: Action::Transform(Transform::Split {
                input: None,
                delimiter: "\n".into(),
            }),
            assign_to: None,
        },
        // take first line
        Step::Action {
            action: Action::Transform(Transform::First { input: None }),
            assign_to: None,
        },
        // split file:line
        Step::Action {
            action: Action::Transform(Transform::Split {
                input: None,
                delimiter: ":".into(),
            }),
            assign_to: None,
        },
        // take file only
        Step::Action {
            action: Action::Transform(Transform::First { input: None }),
            assign_to: Some("file".into()),
        },
    ];

    let mut ctx = Context::new(vec![]);

    let result = exec.run(&steps, &mut ctx);

    assert!(result.success);

    match ctx.vars.get("file") {
        Some(Value::String(s)) => assert_eq!(s, "src/main.rs"),
        _ => panic!("Split/First pipeline failed"),
    }
}
