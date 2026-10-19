//! Host await: a host future parks the VM, `resolve_future` / `fail_future`
//! resume it, and synchronous calls still run while parked

use std::{cell::RefCell, rc::Rc};

use monty::{Embed, HostCtx, HostObject, HostValue, HostVtable};

const KIND_VIEW: u16 = 1;
const KIND_FETCH: u16 = 2;

#[derive(Default)]
struct State {
    call_id: Option<u32>,
}

struct AwaitHost {
    state: Rc<RefCell<State>>,
}

impl HostVtable for AwaitHost {
    fn call_attr(
        &mut self,
        _ctx: &mut dyn HostCtx,
        _id: monty::HeapId,
        _obj: HostObject,
        attr: &str,
        _args: Vec<HostValue>,
        _kwargs: Vec<(String, HostValue)>,
    ) -> Result<HostValue, String> {
        Err(format!("no host method {attr}"))
    }

    fn getattr(
        &mut self,
        _ctx: &mut dyn HostCtx,
        _id: monty::HeapId,
        _obj: HostObject,
        _attr: &str,
    ) -> Result<Option<HostValue>, String> {
        Ok(None)
    }

    fn construct(&mut self, _ctx: &mut dyn HostCtx, name: &str, _args: Vec<HostValue>) -> Result<HostValue, String> {
        Err(format!("no host constructor {name}"))
    }

    fn create_native_module(&mut self, ctx: &mut dyn HostCtx, name: &str) -> Result<Option<monty::HeapId>, String> {
        let (module_name, attr, kind) = match name {
            "ui" => ("ui", "view", KIND_VIEW),
            "net" => ("net", "fetch", KIND_FETCH),
            _ => return Ok(None),
        };
        let module = ctx.alloc_module(module_name)?;
        let handle = ctx.alloc(HostObject { kind, data: 0 });
        ctx.module_set_attr(module, attr, HostValue::Heap(handle))?;
        Ok(Some(module))
    }

    fn call(
        &mut self,
        ctx: &mut dyn HostCtx,
        _id: monty::HeapId,
        obj: HostObject,
        args: Vec<HostValue>,
        _kwargs: Vec<(String, HostValue)>,
    ) -> Result<HostValue, String> {
        match obj.kind {
            KIND_VIEW => {
                let Some(HostValue::Heap(class)) = args.first() else {
                    return Err("@view expects a class".into());
                };
                ctx.mark_view_class(*class)?;
                ctx.inc_ref(*class);
                Ok(HostValue::Heap(*class))
            }
            KIND_FETCH => {
                let (call_id, value) = ctx.host_await()?;
                self.state.borrow_mut().call_id = Some(call_id);
                Ok(value)
            }
            _ => Err(format!("host object kind {} is not callable", obj.kind)),
        }
    }
}

const APP: &str = r#"
from ui import view
from net import fetch

last = ""

@view
class App:
    def init(self, props):
        pass

    async def load(self):
        global last
        try:
            text = await fetch()
            last = "got:" + text
        except Exception as err:
            last = "err:" + str(err)

    def sync(self):
        return 7

    async def tick(self):
        return 1

    def result(self):
        return last
"#;

fn app() -> (Embed, Rc<RefCell<State>>, monty::HeapId) {
    let state = Rc::new(RefCell::new(State::default()));
    let host: Rc<RefCell<dyn HostVtable>> = Rc::new(RefCell::new(AwaitHost {
        state: Rc::clone(&state),
    }));
    let mut embed = Embed::run_source(APP.into(), "main.py", host).expect("compile/run");
    let class = embed.find_view_class().expect("view class");
    let instance = embed.construct_instance(class).expect("instance");
    (embed, state, instance)
}

fn call_id(state: &Rc<RefCell<State>>) -> u32 {
    state.borrow().call_id.expect("fetch asked for a host future")
}

#[test]
fn resolve_future_feeds_the_awaited_value() {
    let (mut embed, state, instance) = app();
    embed.call_method(instance, "load", Vec::new()).expect("load starts");
    assert!(embed.is_parked());

    match embed
        .call_method(instance, "sync", Vec::new())
        .expect("sync while parked")
    {
        HostValue::Int(7) => {}
        other => panic!("expected 7, got {other:?}"),
    }
    assert!(embed.is_parked(), "a sync call leaves the parked task parked");

    embed
        .resolve_future(call_id(&state), HostValue::Str("ok".to_owned()))
        .expect("resolve");
    assert!(!embed.is_parked());

    match embed.call_method(instance, "result", Vec::new()).expect("result") {
        HostValue::Str(text) => assert_eq!(text, "got:ok"),
        other => panic!("expected text, got {other:?}"),
    }
}

#[test]
fn async_call_while_parked_keeps_the_waiting_task() {
    let (mut embed, state, instance) = app();
    embed.call_method(instance, "load", Vec::new()).expect("load starts");
    let id = call_id(&state);
    embed
        .call_method(instance, "tick", Vec::new())
        .expect("tick while parked");
    assert!(embed.is_parked(), "tick must not drop the parked load");

    embed
        .resolve_future(id, HostValue::Str("ok".to_owned()))
        .expect("resolve");
    assert!(!embed.is_parked());

    match embed.call_method(instance, "result", Vec::new()).expect("result") {
        HostValue::Str(text) => assert_eq!(text, "got:ok"),
        other => panic!("expected text, got {other:?}"),
    }
}

#[test]
fn fail_future_is_caught_by_except_exception() {
    let (mut embed, state, instance) = app();
    embed.call_method(instance, "load", Vec::new()).expect("load starts");
    assert!(embed.is_parked());

    embed.fail_future(call_id(&state), "boom").expect("fail");
    assert!(!embed.is_parked());

    match embed.call_method(instance, "result", Vec::new()).expect("result") {
        HostValue::Str(text) => assert_eq!(text, "err:boom"),
        other => panic!("expected text, got {other:?}"),
    }
}
