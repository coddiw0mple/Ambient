#[cfg(feature = "native")]
use std::future::Future;
use std::{
    any::{type_name, TypeId},
    collections::{HashMap, HashSet},
    fmt::Debug,
    sync::Arc,
};

#[cfg(feature = "native")]
use ambient_core::runtime;
#[cfg(feature = "native")]
use ambient_guest_bridge::ecs::{world_events, ComponentQuery, FrameEvent, QueryState, TypedReadQuery};
use ambient_guest_bridge::ecs::{ComponentValue, Entity, World};
#[cfg(feature = "native")]
use ambient_sys::task;
use as_any::Downcast;
#[cfg(feature = "native")]
use atomic_refcell::AtomicRefCell;
use cb::{cb, Cb};
#[cfg(feature = "native")]
use itertools::Itertools;
use parking_lot::Mutex;
#[cfg(feature = "native")]
use tracing::info_span;

use crate::{AnyCloneable, ElementTree, HookContext, InstanceId};

pub type Setter<T> = Cb<dyn Fn(T) + Sync + Send>;

pub type SpawnFn = Box<dyn FnOnce(&mut World) -> DespawnFn + Sync + Send>;
pub type DespawnFn = Box<dyn FnOnce(&mut World) + Sync + Send>;

pub struct Hooks<'a> {
    pub world: &'a mut World,
    pub(crate) tree: &'a mut ElementTree,
    pub(crate) element: InstanceId,
    pub(crate) state_index: usize,
    pub(crate) on_spawn: Option<Vec<SpawnFn>>,
    pub(crate) environment: Arc<Mutex<HooksEnvironment>>,
}

impl<'a> Hooks<'a> {
    pub fn use_state<T: Clone + Debug + ComponentValue + Send + 'static>(&mut self, init: T) -> (T, Setter<T>) {
        self.use_state_with(|_| init)
    }

    pub fn use_state_with<T: Clone + Debug + Send + 'static, F: FnOnce(&mut World) -> T>(&mut self, init: F) -> (T, Setter<T>) {
        let index = self.state_index;
        self.state_index += 1;
        let value = {
            let instance = self.tree.instances.get_mut(&self.element).unwrap();
            if let Some(value) = instance.hooks_state.get(index) {
                value
            } else {
                instance.hooks_state.push(Box::new(init(self.world)));
                instance.hooks_state.last().unwrap()
            }
            .downcast_ref::<T>()
            .unwrap()
            .clone()
        };
        let environment = self.environment.clone();
        let element = self.element.clone();
        (
            value,
            cb(move |new_value| {
                environment.lock().set_states.push(StateUpdate {
                    instance_id: element.clone(),
                    index,
                    value: Box::new(new_value),
                    name: type_name::<T>(),
                })
            }),
        )
    }

    /// Provides a function that, when called, will cause this [Element] to be re-rendered.
    // TODO: consider a more efficient implementation?
    pub fn use_rerender_signal(&mut self) -> Cb<dyn Fn() + Sync + Send> {
        let (_, signal) = self.use_state(());
        cb(move || signal(()))
    }

    /// Provide a value which is accessible to all children further down the
    /// tree.
    ///
    /// **Note**: Does not rely on order, and is therefore safe to use inside
    /// conditionals.
    pub fn provide_context<T: Clone + Debug + ComponentValue + Send + Sync + 'static>(
        &mut self,
        default_value: impl FnOnce() -> T,
    ) -> Setter<T> {
        let instance = self.tree.instances.get_mut(&self.element).unwrap();
        let type_id = TypeId::of::<T>();
        instance
            .hooks_context_state
            .entry(type_id)
            .or_insert_with(|| HookContext { value: Box::new(default_value()), listeners: HashSet::new() });
        let environment = self.environment.clone();
        let element = self.element.clone();
        cb(move |new_value| {
            environment.lock().set_contexts.push(ContextUpdate {
                instance_id: element.clone(),
                type_id: TypeId::of::<T>(),
                name: type_name::<T>(),
                value: Box::new(new_value),
            });
        })
    }
    #[allow(clippy::type_complexity)]
    pub fn consume_context<T: Clone + Debug + ComponentValue + Sync + Send + 'static>(&mut self) -> Option<(T, Setter<T>)> {
        let type_id = TypeId::of::<T>();
        if let Some(provider) = self.tree.get_context_provider(&self.element, type_id) {
            let value = {
                let instance = self.tree.instances.get_mut(&provider).unwrap();
                let ctx = instance.hooks_context_state.get_mut(&type_id).unwrap();
                ctx.listeners.insert(self.element.clone());
                ctx.value.downcast_ref::<T>().unwrap().clone()
            };
            {
                let instance = self.tree.instances.get_mut(&self.element).unwrap();
                instance.hooks_context_listening.insert((provider.clone(), type_id));
            }
            let environment = self.environment.clone();
            Some((
                value,
                cb(move |new_value| {
                    environment.lock().set_contexts.push(ContextUpdate {
                        instance_id: provider.clone(),
                        type_id,
                        name: type_name::<T>(),
                        value: Box::new(new_value),
                    });
                }),
            ))
        } else {
            None
        }
    }

    /// Execute a function upon the world the first time is mounted, E.g;
    /// rendered.
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn use_spawn<F: FnOnce(&mut World) -> DespawnFn + Sync + Send + 'static>(&mut self, func: F) {
        if let Some(ref mut on_spawn) = self.on_spawn {
            on_spawn.push(Box::new(func) as SpawnFn);
        }
    }

    pub fn use_world_event(&mut self, func: impl Fn(&mut World, &Entity) + Sync + Send + 'static) {
        #[cfg(feature = "native")]
        {
            let reader = self.use_ref_with(|world| world.resource(world_events()).reader());
            self.use_frame(move |world| {
                let mut reader = reader.lock();
                let events = reader.iter(world.resource(world_events())).map(|(_, event)| event.clone()).collect_vec();
                for event in events {
                    func(world, &event);
                }
            })
        }
        #[cfg(feature = "guest")]
        {
            ambient_guest_bridge::api::global::on(ambient_guest_bridge::api::event::WORLD_EVENT, move |data| {
                func(&mut World, data);
                ambient_guest_bridge::api::prelude::EventOk
            });
        }
    }

    /// Spawns the provided future as a task, and aborts the task when the
    /// entity is despawned.
    #[cfg(feature = "native")]
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn use_task<T: Future<Output = ()> + Send + 'static>(&mut self, task: impl FnOnce(&mut World) -> T + Send + Sync + 'static) {
        if let Some(ref mut on_spawn) = self.on_spawn {
            let spawn = Box::new(move |w: &mut World| {
                let task = task(w);
                let task = w.resource(runtime()).spawn(task);
                Box::new(move |_: &mut World| task.abort()) as Box<dyn FnOnce(&mut World) + Sync + Send>
            });

            on_spawn.push(spawn)
        }
    }

    /// Use state dependent on a future
    #[cfg(feature = "native")]
    pub fn use_async<T, U>(&mut self, future: impl FnOnce(&mut World) -> T + Send + Sync + 'static) -> Option<U>
    where
        T: Future<Output = U> + Send + 'static,
        U: Debug + ComponentValue,
    {
        let (state, set_state) = self.use_state(None);
        self.use_spawn(|w| {
            let future = future(w);
            let runtime = w.resource(runtime());

            info_span!("use_async_spawn");
            let task = runtime.spawn(async move {
                let res = future.await;
                set_state(Some(res))
            });

            Box::new(move |_| task.abort())
        });

        state
    }

    /// Use memoized state dependent on a future
    #[cfg(feature = "native")]
    pub fn use_memo_async<T, F, D, U>(&mut self, deps: D, init: F) -> Option<T>
    where
        F: FnOnce(&mut World, D) -> U,
        U: 'static + Future<Output = T> + Send,
        T: ComponentValue,
        D: PartialEq + ComponentValue,
    {
        struct State<T, D> {
            task: AtomicRefCell<Option<task::JoinHandle<()>>>,
            value: Mutex<Option<T>>,
            prev_deps: AtomicRefCell<Option<D>>,
        }

        impl<T, D> Debug for State<T, D> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct("State")
                    .field("task", &"...")
                    .field("value", &type_name::<T>())
                    .field("prev_deps", &type_name::<D>())
                    .finish()
            }
        }

        let (state, set_state) = self.use_state_with(|_| {
            Arc::new(State { task: AtomicRefCell::new(None), value: Mutex::new(None), prev_deps: AtomicRefCell::new(None) })
        });

        let mut prev_deps = state.prev_deps.borrow_mut();
        if prev_deps.as_ref() != Some(&deps) {
            let runtime = self.world.resource(runtime()).clone();
            // Update state
            // Cancel the previous calculation
            let mut task = state.task.borrow_mut();
            if let Some(task) = task.take() {
                task.abort();
            }

            let fut = init(self.world, deps.clone());
            *prev_deps = Some(deps);
            let state = state.clone();
            // The future may complete immediately
            *task = Some(runtime.spawn(async move {
                let value = fut.await;

                // Update the value
                *state.value.lock() = Some(value);
                set_state(state)
            }));
        } else {
            // A value may be available, but nothing is certain
        }

        let x = state.value.lock().as_ref().cloned();
        x
    }

    #[profiling::function]
    pub fn use_frame<F: Fn(&mut World) + Sync + Send + 'static>(&mut self, on_frame: F) {
        let mut env = self.environment.lock();
        let listeners = env.frame_listeners.entry(self.element.clone()).or_insert_with(Vec::new);
        listeners.push(FrameListener(Arc::new(on_frame)));
    }

    // Helpers
    pub fn use_ref_with<T: Send + Debug + 'static>(&mut self, init: impl FnOnce(&mut World) -> T) -> Arc<Mutex<T>> {
        self.use_state_with(|world| Arc::new(Mutex::new(init(world)))).0
    }

    #[profiling::function]
    pub fn use_memo_with<
        T: Clone + ComponentValue + Debug + Sync + Send + 'static,
        D: PartialEq + Clone + Sync + Send + Debug + 'static,
    >(
        &mut self,
        dependencies: D,
        create: impl FnOnce(&mut World, &D) -> T,
    ) -> T {
        let value = self.use_ref_with(|_| None);
        let prev_deps = self.use_ref_with(|_| None);

        let mut prev_deps = prev_deps.lock();
        let mut value = value.lock();

        if prev_deps.as_ref() != Some(&dependencies) {
            let value = value.insert(create(self.world, &dependencies)).clone();
            *prev_deps = Some(dependencies);
            value
        } else {
            value.clone().expect("No memo value")
        }
    }

    #[profiling::function]
    /// Run a function for its side effects each time a dependency changes.
    ///
    /// The provided functions returns a function which is run when the part is
    /// removed or `use_effect` is run again.
    ///
    pub fn use_effect<D: PartialEq + ComponentValue + Debug + Sync + Send + 'static>(
        &mut self,
        dependencies: D,
        run: impl FnOnce(&mut World, &D) -> Box<dyn FnOnce(&mut World) + Sync + Send> + Sync + Send,
    ) {
        struct Cleanup(Box<dyn FnOnce(&mut World) + Sync + Send>);
        impl Debug for Cleanup {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_tuple("Cleanup").finish()
            }
        }
        let cleanup_prev: Arc<Mutex<Option<Cleanup>>> = self.use_ref_with(|_| None);
        let prev_deps = self.use_ref_with::<Option<D>>(|_| None);
        {
            let cleanup_prev = cleanup_prev.clone();
            self.use_spawn(move |_| {
                Box::new(move |world| {
                    let mut cleanup_prev = cleanup_prev.lock();
                    if let Some(cleanup_prev) = cleanup_prev.take() {
                        cleanup_prev.0(world);
                    }
                })
            });
        }

        let dependencies = dependencies;
        let mut prev_deps = prev_deps.lock();
        if prev_deps.as_ref() != Some(&dependencies) {
            let mut cleanup_prev = cleanup_prev.lock();
            if let Some(cleanup_prev) = std::mem::replace(&mut *cleanup_prev, None) {
                cleanup_prev.0(self.world);
            }
            profiling::scope!("use_effect_run");
            *cleanup_prev = Some(Cleanup(run(self.world, &dependencies)));
            *prev_deps = Some(dependencies);
        }
    }

    #[cfg(feature = "native")]
    pub fn use_system<
        'b,
        R: ComponentQuery<'b> + Clone + 'static,
        F: Fn(&TypedReadQuery<R>, &mut World, Option<&mut QueryState>, &FrameEvent) + Send + Sync + 'static,
    >(
        &mut self,
        query: TypedReadQuery<R>,
        run: F,
    ) {
        let query_state = self.use_ref_with(|_| QueryState::new());
        self.use_frame(move |world| {
            let mut qs = query_state.lock();
            run(&query, world, Some(&mut qs), &FrameEvent);
        });
    }
}

#[derive(Clone)]
pub(crate) struct FrameListener(pub Arc<dyn Fn(&mut World) + Sync + Send>);
impl Debug for FrameListener {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("FrameListener").finish()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ContextUpdate {
    pub instance_id: InstanceId,
    pub type_id: TypeId,
    pub name: &'static str,
    pub value: Box<dyn AnyCloneable + Sync + Send>,
}

#[derive(Debug, Clone)]
pub(crate) struct StateUpdate {
    pub instance_id: InstanceId,
    pub index: usize,
    pub name: &'static str,
    pub value: Box<dyn AnyCloneable + Send>,
}

#[derive(Debug)]
pub(crate) struct HooksEnvironment {
    pub(crate) set_states: Vec<StateUpdate>,
    /// Pending updates to contexts.
    ///
    /// This is modified through the returned `Setter` closure
    pub(crate) set_contexts: Vec<ContextUpdate>,
    pub(crate) frame_listeners: HashMap<InstanceId, Vec<FrameListener>>,
}
impl HooksEnvironment {
    pub(crate) fn new() -> Self {
        Self { set_states: Vec::new(), set_contexts: Vec::new(), frame_listeners: HashMap::new() }
    }
    pub fn on_element_removed(&mut self, instance_id: &str) {
        self.frame_listeners.remove(instance_id);
    }
}
