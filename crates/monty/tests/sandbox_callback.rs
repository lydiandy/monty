//! Sandbox functions cross to the host as handles and can be called back.

use monty::{MontyRepl, ReplProgress};
use monty_types::{CompileOptions, MontyObject, PrintWriter, ResourceTracker};

/// Drives a progress to completion, answering no host calls.
fn complete(progress: ReplProgress) -> (MontyRepl, MontyObject) {
    let mut progress = progress;
    loop {
        match progress {
            ReplProgress::Complete { repl, value } => return (repl, value),
            ReplProgress::FunctionCall(call) => {
                progress = call.resume(MontyObject::none(), PrintWriter::Stdout).expect("resume");
            }
            ReplProgress::OsCall(call) => {
                progress = call.resume(MontyObject::none(), PrintWriter::Stdout).expect("resume");
            }
            ReplProgress::NameLookup(call) => {
                progress = call.resume(None.into(), PrintWriter::Stdout).expect("resume");
            }
            ReplProgress::ResolveFutures(_) => panic!("unexpected futures"),
        }
    }
}

fn repl() -> MontyRepl {
    MontyRepl::new("callback.py", ResourceTracker::default(), CompileOptions::default())
}

#[test]
fn a_plain_function_crosses_as_a_handle_and_calls_back() {
    let mut repl = repl();
    let callback = repl
        .feed_run(
            "def add_one(x):\n    return x + 1\nadd_one",
            vec![],
            PrintWriter::Stdout,
        )
        .expect("the function");
    assert!(
        callback.as_sandbox_function().is_some(),
        "a sandbox function crosses as a handle: {callback}"
    );
    let progress = repl
        .call_callback_start(callback, vec![MontyObject::int(41)], PrintWriter::Stdout)
        .expect("the call");
    let (_repl, value) = complete(progress);
    assert_eq!(value, MontyObject::int(42));
}

#[test]
fn a_class_crosses_as_a_handle_and_comes_back() {
    let mut repl = repl();
    let class = repl
        .feed_run(
            "class Thing:\n    def value(self):\n        return 7\nThing",
            vec![],
            PrintWriter::Stdout,
        )
        .expect("the class");
    assert!(
        class.as_sandbox_class().is_some(),
        "a sandbox class crosses as a handle: {class}"
    );
    let value = repl
        .feed_run("cls().value()", vec![("cls".to_owned(), class)], PrintWriter::Stdout)
        .expect("the instance");
    assert_eq!(value, MontyObject::int(7));
}

#[test]
fn a_closure_calls_back() {
    let mut repl = repl();
    let callback = repl
        .feed_run(
            "def make(n):\n    def add(x):\n        return x + n\n    return add\nmake(41)",
            vec![],
            PrintWriter::Stdout,
        )
        .expect("the closure");
    assert!(callback.as_sandbox_function().is_some());
    let progress = repl
        .call_callback_start(callback, vec![MontyObject::int(1)], PrintWriter::Stdout)
        .expect("the call");
    let (_repl, value) = complete(progress);
    assert_eq!(value, MontyObject::int(42));
}

#[test]
fn a_bound_method_calls_back() {
    let mut repl = repl();
    let callback = repl
        .feed_run(
            "class Counter:\n    def __init__(self, n):\n        self.n = n\n    def add(self, x):\n        return self.n + x\nCounter(40).add",
            vec![],
            PrintWriter::Stdout,
        )
        .expect("the bound method");
    assert!(callback.as_sandbox_function().is_some());
    let progress = repl
        .call_callback_start(callback, vec![MontyObject::int(2)], PrintWriter::Stdout)
        .expect("the call");
    let (_repl, value) = complete(progress);
    assert_eq!(value, MontyObject::int(42));
}

#[test]
fn a_callback_can_call_a_host_function_while_suspended() {
    let mut repl = MontyRepl::new("callback.py", ResourceTracker::default(), CompileOptions::default());
    // The callback names a host function; `call_callback_start` suspends on it.
    let callback = repl
        .feed_run(
            "def plus_one(x):\n    return add(x, 1)\nplus_one",
            vec![],
            PrintWriter::Stdout,
        )
        .expect("the function");
    let mut progress = repl
        .call_callback_start(callback, vec![MontyObject::int(41)], PrintWriter::Stdout)
        .expect("the call");
    let mut saw_call = false;
    loop {
        match progress {
            ReplProgress::Complete { value, .. } => {
                assert!(saw_call);
                assert_eq!(value, MontyObject::int(42));
                break;
            }
            ReplProgress::NameLookup(call) => {
                progress = call
                    .resume(Some(MontyObject::function("add", None)).into(), PrintWriter::Stdout)
                    .expect("resume lookup");
            }
            ReplProgress::FunctionCall(call) => {
                assert_eq!(call.function_name, "add");
                saw_call = true;
                progress = call
                    .resume(MontyObject::int(42), PrintWriter::Stdout)
                    .expect("resume call");
            }
            ReplProgress::OsCall(call) => {
                progress = call
                    .resume(MontyObject::none(), PrintWriter::Stdout)
                    .expect("resume os");
            }
            ReplProgress::ResolveFutures(_) => panic!("unexpected futures"),
        }
    }
}

#[test]
fn a_callback_handle_can_be_passed_back_in() {
    let mut repl = MontyRepl::new("callback.py", ResourceTracker::default(), CompileOptions::default());
    let callback = repl
        .feed_run("def identity(x):\n    return x\nidentity", vec![], PrintWriter::Stdout)
        .expect("the function");
    let id = callback.as_sandbox_function().expect("a handle");
    let back = repl
        .feed_run("f", vec![("f".to_owned(), callback.clone())], PrintWriter::Stdout)
        .expect("the value back");
    assert_eq!(back.as_sandbox_function(), Some(id), "one handle per function");
    assert!(callback.as_sandbox_function().is_some());
}
