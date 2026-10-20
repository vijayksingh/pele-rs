//! Extensible host records stay live across growth and synchronized replacement.
use pele::compat::{public::*, text::MetricsResources, *};
use std::rc::Rc;

const NOW: f64 = 1_704_067_200_000.;
const NAN_BITS: u64 = 0x7ff8_0000_0000_0042;

fn number(value: Atom) -> u64 {
    let Atom::Number(value) = value else {
        panic!("Expected numeric host field")
    };
    value.to_bits()
}

struct ExtendRecord;
impl HostFunction for ExtendRecord {
    fn call(
        &self,
        model: &mut dyn ActiveModel,
        receiver: Atom,
        arguments: &[Atom],
        resources: &mut dyn RenderResources,
    ) -> Result<Atom, PublicError> {
        let tasks = model.invoke_builtin(
            MethodCall {
                method: MethodId(19),
                receiver: receiver.clone(),
                arguments: arguments.to_vec(),
            },
            resources,
        )?;
        model.begin_sync()?;
        let record = model.object()?;
        for index in 0..40 {
            let key = format!("field{index}");
            model.set(&record, &key, Atom::Number(index as f64))?;
            assert_eq!(number(model.read(&record, &key)?), (index as f64).to_bits());
        }
        // Every original field remains readable after growing the same object.
        for index in (0..40).rev() {
            let key = format!("field{index}");
            model.set(&record, &key, Atom::Number(index as f64 + 100.))?;
            assert_eq!(
                number(model.read(&record, &key)?),
                (index as f64 + 100.).to_bits()
            );
        }
        let Atom::String(locked) = model.intern(JsString::from("locked"))? else {
            panic!("Intern must return a string")
        };
        model.set(&receiver, "extensionRecord", record.clone())?;
        let published = model.take_delta()?;
        let Atom::Ref(record_id) = record else {
            panic!("Record must retain identity")
        };
        let original = published
            .nodes
            .iter()
            .find(|(id, _)| *id == record_id)
            .unwrap()
            .1
            .properties()[0]
            .key;
        let strings_start = published.strings_start + published.strings.len() as u32;
        let nodes_start = published.nodes_start
            + published
                .nodes
                .iter()
                .filter(|(id, _)| *id >= published.nodes_start)
                .count() as u32;
        // A host replaces a large record with a small one and a duplicate string
        // ID. The first canonical key keeps its descriptor and numeric bits.
        model.apply_delta(ModelDelta {
            strings_start,
            strings: vec![JsString::from("field0")],
            nodes_start,
            nodes: vec![(
                record_id,
                Node::Object {
                    properties: vec![
                        Property {
                            key: strings_start,
                            value: Atom::Number(-0.),
                            flags: 7,
                        },
                        Property {
                            key: original,
                            value: Atom::Number(999.),
                            flags: 0,
                        },
                        Property {
                            key: locked,
                            value: Atom::Number(f64::from_bits(NAN_BITS)),
                            flags: 0,
                        },
                    ],
                },
            )],
        })?;
        let record = Atom::Ref(record_id);
        assert_eq!(number(model.read(&record, "field0")?), (-0f64).to_bits());
        assert!(matches!(model.read(&record, "field39")?, Atom::Undefined));
        model.set(&record, "field0", Atom::Number(42.))?;
        assert_eq!(number(model.read(&record, "field0")?), 42f64.to_bits());
        assert_eq!(number(model.read(&record, "locked")?), NAN_BITS);
        let error = model.set(&record, "locked", Atom::Number(0.)).unwrap_err();
        assert!(matches!(error, PublicError::JavaScript {
            name: JavaScriptErrorKind::TypeError, message,
        } if message.is("Cannot assign to read only property 'locked' of object '#<Object>'")));
        assert_eq!(number(model.read(&record, "locked")?), NAN_BITS);
        // Grow again in the same callback after host synchronization.
        for index in 1..40 {
            model.set(
                &record,
                &format!("field{index}"),
                Atom::Number(index as f64),
            )?;
        }
        assert_eq!(number(model.read(&record, "field0")?), 42f64.to_bits());
        assert_eq!(number(model.read(&record, "field39")?), 39f64.to_bits());
        assert_eq!(number(model.read(&record, "locked")?), NAN_BITS);
        Ok(tasks)
    }
}

struct Host;
impl RenderResources for Host {
    fn host_function(&mut self, id: u32) -> Result<Rc<dyn HostFunction>, PublicError> {
        assert_eq!(id, 7);
        Ok(Rc::new(ExtendRecord))
    }
    fn text_width(&mut self, request: TextRequest<'_>) -> Result<f64, PeleError> {
        MetricsResources.text_width(request)
    }
}

#[test]
fn host_record_growth_replacement_and_descriptors_survive_rendering() {
    let source = "gantt\ndateFormat YYYY-MM-DD\ntodayMarker off\nA:a,2024-01-01,1d\nB:b,after a,2d";
    let mut options = RenderOptions::default();
    options.render.now = Some(NOW);
    let expected = Engine::with_builtins()
        .render_source(source, &options, &mut MetricsResources)
        .unwrap()
        .rendered;
    let mut document = PublicEngine::with_builtins()
        .parse(source, &options.parse)
        .unwrap();
    let mut graph = document.snapshot().unwrap();
    let Atom::Ref(root) = graph.root else {
        panic!("Expected Gantt instance")
    };
    let key = graph.strings.len() as u32;
    graph.strings.push(JsString::from("getTasks"));
    graph.nodes[root as usize].properties_mut().push(Property {
        key,
        value: Atom::HostReference(7),
        flags: 7,
    });
    document.apply(0, graph).unwrap();
    for revision in 1..=2 {
        let outcome = document
            .render_operation(revision, &options, &mut Host)
            .unwrap();
        assert_eq!(outcome.revision, revision + 1);
        assert_eq!(outcome.result.unwrap().rendered, expected);
        let Atom::Ref(record) = outcome.graph.field("extensionRecord").unwrap().unwrap() else {
            panic!("Published helper record must retain identity")
        };
        let properties = outcome.graph.nodes[*record as usize].properties();
        assert_eq!(properties.len(), 42);
        assert_eq!(number(properties[0].value.clone()), 42f64.to_bits());
        assert_eq!(number(properties[1].value.clone()), 999f64.to_bits());
        assert_eq!(properties[1].flags, 0);
        assert_eq!(number(properties[2].value.clone()), NAN_BITS);
        assert_eq!(properties[2].flags, 0);
    }
}
