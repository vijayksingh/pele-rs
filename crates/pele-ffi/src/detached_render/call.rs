//! Short owner transactions; neither model nor binding guards cross host calls.
use super::*;
use pele::compat::{
    public::{
        ActiveModel, DetachedInvocationStep, MethodCall, MethodId, PublicActiveBudget,
        PublicActiveOperation as Op, PublicActiveValue as Out, PublicHostOperation,
        PublicHostResult, PublicHostValue,
    },
    RenderResources,
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
#[derive(Clone)]
pub(super) struct Access {
    budget: Rc<RefCell<PublicActiveBudget>>,
    depth: Rc<Cell<usize>>,
}
impl Default for Access {
    fn default() -> Self {
        Self {
            budget: Rc::new(RefCell::new(PublicActiveBudget::default())),
            depth: Rc::new(Cell::new(0)),
        }
    }
}
fn access(owner: *const PelePublicDocument, scope: &Access, op: Op) -> Result<Out, PublicError> {
    let mut budget = scope
        .budget
        .try_borrow_mut()
        .map_err(|_| invalid("Active budget is already in use"))?;
    let mut doc = unsafe { public::document(owner) }?;
    doc.active_operation(op, &mut budget)
}
fn atom(owner: *const PelePublicDocument, scope: &Access, op: Op) -> Result<Atom, PublicError> {
    match access(owner, scope, op)? {
        Out::Atom(v) => Ok(v),
        Out::String(_) => Err(invalid("Invalid active operation result").into()),
    }
}
struct Active {
    owner: *const PelePublicDocument,
    scope: Access,
    root: Atom,
}
impl ActiveModel for Active {
    fn root(&self) -> Atom {
        self.root.clone()
    }
    fn validate(&self, v: &Atom) -> Result<(), PublicError> {
        access(self.owner, &self.scope, Op::Validate(v.clone())).map(|_| ())
    }
    fn read(&mut self, r: &Atom, k: &str) -> Result<Atom, PublicError> {
        atom(
            self.owner,
            &self.scope,
            Op::Read {
                receiver: r.clone(),
                key: k.into(),
            },
        )
    }
    fn set(&mut self, r: &Atom, k: &str, v: Atom) -> Result<(), PublicError> {
        atom(
            self.owner,
            &self.scope,
            Op::Set {
                receiver: r.clone(),
                key: k.into(),
                value: v,
            },
        )
        .map(|_| ())
    }
    fn intern(&mut self, v: JsString) -> Result<Atom, PublicError> {
        atom(self.owner, &self.scope, Op::Intern(v))
    }
    fn string(&mut self, v: &Atom) -> Result<JsString, PublicError> {
        match access(self.owner, &self.scope, Op::String(v.clone()))? {
            Out::String(s) => Ok(s),
            _ => Err(invalid("Invalid active string result").into()),
        }
    }
    fn object(&mut self) -> Result<Atom, PublicError> {
        atom(self.owner, &self.scope, Op::Object)
    }
    fn array(&mut self) -> Result<Atom, PublicError> {
        atom(self.owner, &self.scope, Op::Array)
    }
    fn date(&mut self, n: f64) -> Result<Atom, PublicError> {
        atom(self.owner, &self.scope, Op::Date(n))
    }
    fn date_millis(&mut self, v: &Atom) -> Result<f64, PublicError> {
        match atom(self.owner, &self.scope, Op::DateMillis(v.clone()))? {
            Atom::Number(n) => Ok(n),
            _ => Err(invalid("Invalid active Date result").into()),
        }
    }
    fn push(&mut self, r: &Atom, v: Atom) -> Result<(), PublicError> {
        atom(
            self.owner,
            &self.scope,
            Op::Push {
                receiver: r.clone(),
                value: v,
            },
        )
        .map(|_| ())
    }
    fn invoke_builtin(
        &mut self,
        call: MethodCall,
        host: &mut dyn RenderResources,
    ) -> Result<Atom, PublicError> {
        let mut reader = Reader::for_call(self.owner, self.scope.clone())?;
        reader.invoke_owned(call, host)
    }
}
struct Depth(Rc<Cell<usize>>);
impl Drop for Depth {
    fn drop(&mut self) {
        self.0.set(self.0.get() - 1)
    }
}
impl Reader {
    fn for_call(owner: *const PelePublicDocument, access: Access) -> Result<Self, PublicError> {
        let root = {
            let doc = unsafe { public::document(owner) }?;
            let graph = doc.snapshot_read()?;
            own(graph.reader(), graph.reader().root())?
        };
        Ok(Self {
            owner,
            root,
            identities: HashMap::new(),
            iterator_handles: 0,
            iterator_advances: 0,
            access,
        })
    }
    pub(super) fn date_argument(&self, milliseconds: f64) -> Result<Value, PublicError> {
        atom(self.owner, &self.access, Op::Date(milliseconds)).map(Value::Atom)
    }
    pub(super) fn record_argument(
        &self,
        fields: &[(JsString, Value)],
    ) -> Result<Value, PublicError> {
        let mut values = Vec::with_capacity(fields.len());
        for (key, value) in fields {
            values.push((key.clone(), self.argument_atom(value)?));
        }
        atom(self.owner, &self.access, Op::Record(values)).map(Value::Atom)
    }
    pub(super) fn argument_atom(&self, v: &Value) -> Result<Atom, PublicError> {
        match v {
            Value::Atom(v) => Ok(v.clone()),
            Value::String(s) => atom(self.owner, &self.access, Op::Intern((**s).clone())),
            _ => Err(invalid("Native live calls require graph-coordinate arguments").into()),
        }
    }
    pub(super) fn call_owned(
        &mut self,
        f: &Value,
        receiver: &Value,
        args: &[Value],
        host: &mut dyn RenderResources,
        message: &str,
    ) -> Result<Value, PublicError> {
        self.call_owned_policy(f, receiver, args, host, message, PublicHostResult::Value)
    }
    fn call_owned_policy(
        &mut self,
        f: &Value,
        receiver: &Value,
        args: &[Value],
        host: &mut dyn RenderResources,
        message: &str,
        policy: PublicHostResult,
    ) -> Result<Value, PublicError> {
        if self.access.depth.get() >= 64 {
            return Err(
                PeleError::new("Public model callback depth exceeded", ErrorCode::Limit).into(),
            );
        }
        self.access.depth.set(self.access.depth.get() + 1);
        let _depth = Depth(self.access.depth.clone());
        let units = args
            .iter()
            .chain(std::iter::once(receiver))
            .fold(0usize, |total, v| {
                total.saturating_add(if let Value::String(s) = v {
                    s.0.len()
                } else {
                    0
                })
            });
        if args.len() > 1_000_000 || units > 16_000_000 {
            return Err(
                PeleError::new("Live call argument budget exceeded", ErrorCode::Limit).into(),
            );
        }
        if matches!(f, Value::Atom(Atom::HostReference(_))) {
            let mut values = Vec::with_capacity(args.len() + 2);
            values.push(f);
            values.push(receiver);
            values.extend(args);
            if let Some(value) = self.opaque_operation(1, &values, "", message, policy)? {
                return Ok(value);
            }
        }
        let r = self.argument_atom(receiver)?;
        let args = args
            .iter()
            .map(|v| self.argument_atom(v))
            .collect::<Result<Vec<_>, _>>()?;
        let value = match f {
            Value::Atom(Atom::BuiltinFunction(id)) => self.invoke_owned(
                MethodCall {
                    method: MethodId(*id),
                    receiver: r,
                    arguments: args,
                },
                host,
            )?,
            Value::Atom(Atom::HostReference(id)) => {
                let function = host.host_function(*id)?;
                let root = {
                    let doc = unsafe { public::document(self.owner) }?;
                    doc.snapshot_read()?.reader().root()
                };
                let mut active = Active {
                    owner: self.owner,
                    scope: self.access.clone(),
                    root,
                };
                let value = function.call(&mut active, r, &args, host)?;
                active.validate(&value)?;
                value
            }
            _ => return Err(type_error(message.into())),
        };
        let doc = unsafe { public::document(self.owner) }?;
        let graph = doc.snapshot_read()?;
        Ok(own(graph.reader(), value)?)
    }
    fn invoke_owned(
        &mut self,
        call: MethodCall,
        host: &mut dyn RenderResources,
    ) -> Result<Atom, PublicError> {
        let job = {
            let mut doc = unsafe { public::document(self.owner) }?;
            let rev = doc.revision();
            doc.prepare_detached_invocation(rev, call.clone())?
        };
        let Some(mut job) = job else {
            return atom(self.owner, &self.access, Op::Builtin(call));
        };
        let mut reply = None;
        loop {
            let step = {
                let mut doc = unsafe { public::document(self.owner) }?;
                doc.resume_detached_invocation(&mut job, reply.take())?
            };
            let op = match step {
                DetachedInvocationStep::Complete => break,
                DetachedInvocationStep::Resource(operation) => {
                    reply = Some(operation.run(host));
                    continue;
                }
                DetachedInvocationStep::Host(op) => op,
            };
            let owner = self.owner;
            let resolve = |a: Atom| -> Result<Value, PublicError> {
                let doc = unsafe { public::document(owner) }?;
                let graph = doc.snapshot_read()?;
                Ok(own(graph.reader(), a)?)
            };
            let result = (|| -> Result<(Value, PublicHostResult), PublicError> {
                match op {
                    PublicHostOperation::Get { receiver, key } => {
                        let value = resolve(receiver)?;
                        Ok((self.field(&value, key)?, PublicHostResult::Value))
                    }
                    PublicHostOperation::Call {
                        function,
                        receiver,
                        arguments,
                        not_callable_error,
                        result,
                    } => {
                        let function = resolve(function)?;
                        let receiver = resolve(receiver)?;
                        let arguments = arguments
                            .into_iter()
                            .map(resolve)
                            .collect::<Result<Vec<_>, _>>()?;
                        self.call_owned_policy(
                            &function,
                            &receiver,
                            &arguments,
                            host,
                            not_callable_error,
                            result,
                        )
                        .map(|v| (v, result))
                    }
                    PublicHostOperation::Truthy { value } => {
                        let value = resolve(value)?;
                        let truth = if matches!(value, Value::Atom(Atom::HostReference(_))) {
                            self.opaque_operation(2, &[&value], "", "", PublicHostResult::Value)?
                                .ok_or_else(|| {
                                    invalid("No owner dispatcher for opaque truthiness")
                                })?
                        } else {
                            Value::Atom(Atom::Bool(self.truthy(&value)))
                        };
                        Ok((truth, PublicHostResult::Value))
                    }
                    PublicHostOperation::ToString { value } => {
                        let value = resolve(value)?;
                        let text = self.to_string(&value)?;
                        Ok((Value::String(Arc::new(text)), PublicHostResult::Value))
                    }
                    PublicHostOperation::GreaterEqual { left, right } => {
                        let code = 4;
                        let left = resolve(left)?;
                        let right = resolve(right)?;
                        let result = self
                            .opaque_operation(
                                code,
                                &[&left, &right],
                                "",
                                "",
                                PublicHostResult::Value,
                            )?
                            .ok_or_else(|| invalid("No owner dispatcher for host comparison"))?;
                        Ok((result, PublicHostResult::Value))
                    }
                    PublicHostOperation::StrictEqual { left, right } => {
                        let left = resolve(left)?;
                        let right = resolve(right)?;
                        let result = self
                            .opaque_operation(5, &[&left, &right], "", "", PublicHostResult::Value)?
                            .ok_or_else(|| invalid("No owner dispatcher for host comparison"))?;
                        Ok((result, PublicHostResult::Value))
                    }
                }
            })();
            reply = Some(result.and_then(|(value, policy)| {
                if matches!(policy, PublicHostResult::Discard) {
                    return Ok(PublicHostValue::Undefined);
                }
                if !matches!(policy, PublicHostResult::Retain) {
                    match &value {
                        Value::Atom(Atom::Undefined) => return Ok(PublicHostValue::Undefined),
                        Value::Atom(Atom::Null) => return Ok(PublicHostValue::Null),
                        Value::Atom(Atom::Bool(v)) => return Ok(PublicHostValue::Bool(*v)),
                        Value::Atom(Atom::Number(v)) => return Ok(PublicHostValue::Number(*v)),
                        Value::String(s) => return Ok(PublicHostValue::String((**s).clone())),
                        _ => {}
                    }
                }
                match value {
                    Value::Atom(atom) => Ok(PublicHostValue::Graph(atom)),
                    Value::String(s) => Ok(PublicHostValue::String((*s).clone())),
                    _ => Err(invalid("Native method reply is not a graph-coordinate value").into()),
                }
            }));
        }
        let mut doc = unsafe { public::document(self.owner) }?;
        let outcome = doc.finish_detached_invocation(&mut job)?;
        outcome.result
    }
}
