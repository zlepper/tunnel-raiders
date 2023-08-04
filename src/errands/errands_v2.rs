use bevy::prelude::*;
use bevy_ecs::system::EntityCommands;
use itertools::Itertools;
use std::any::TypeId;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock, TryLockError, Weak};

pub trait Errand: Debug + Clone + Send + Sync + 'static {
    type WorkerComponent: Component;

    fn on_enqueued<TEnqueued: QueuedErrand>(&self, _queued: &mut TEnqueued) {
        // Default implementation does nothing
    }

    fn get_errand_type_order() -> i32;
}

#[derive(Component)]
pub struct ErrandQueue {
    errands: VecDeque<Box<dyn QueuedErrand>>,
    next_id: u64,
}

impl ErrandQueue {
    pub fn new() -> Self {
        Self {
            errands: VecDeque::new(),
            next_id: 1,
        }
    }

    pub fn append_errand<T: QueuedErrand>(&mut self, create: impl FnOnce(u64) -> T) {
        let id = self.next_id;
        self.next_id += 1;

        let queued_errand = create(id);

        info!("Appending errand");

        self.errands.push_back(Box::new(queued_errand));
    }

    pub fn append_independent_errand(&mut self, errand: impl Errand) {
        self.append_errand(|id| QueuedErrandImpl::new(id, errand));
    }

    pub fn prepend_errand<T: QueuedErrand>(&mut self, create: impl FnOnce(u64) -> T) {
        let id = self.next_id;
        self.next_id += 1;

        let queued_errand = create(id);

        self.errands.push_front(Box::new(queued_errand));
    }

    pub fn clear(&mut self) {
        self.errands.clear();
    }

    pub fn len(&self) -> usize {
        self.errands.len()
    }
}

pub trait QueuedErrand: Send + Sync + 'static {
    fn id(&self) -> u64;
    fn activate(&self, commands: &mut EntityCommands);
    fn deactivate(&self, commands: &mut EntityCommands);
    fn fail_on(&self) -> &Vec<FailureCondition>;
    fn add_failure_condition(&mut self, condition: FailureCondition);
}

pub trait QueuedErrandFailureBuilder: QueuedErrand {
    fn fail_if_entity_missing(&mut self, entity: Entity) {
        self.add_failure_condition(FailureCondition::TargetRemoved(entity));
    }

    fn fail_if_cancelled(&mut self, cancelled: Arc<AtomicBool>) {
        self.add_failure_condition(FailureCondition::TargetErrandCancelled(cancelled));
    }
}

impl<T: QueuedErrand> QueuedErrandFailureBuilder for T {}

pub struct QueuedErrandImpl<T: Errand> {
    id: u64,
    errand: T,
    reservation: Option<Arc<ReservedErrand>>,
    fail_on: Vec<FailureCondition>,
}

impl<T: Errand> QueuedErrandImpl<T> {
    pub fn new(id: u64, errand: T) -> Self {
        Self {
            id,
            errand,
            reservation: None,
            fail_on: Vec::new(),
        }
    }
}

impl<T: Errand> QueuedErrand for QueuedErrandImpl<T> {
    fn id(&self) -> u64 {
        self.id
    }

    fn activate(&self, commands: &mut EntityCommands) {
        let work = WorkingOnErrand {
            id: self.id,
            errand: self.errand.clone(),
            _reservation: self.reservation.clone(),
            is_done: false,
            failed: false,
        };

        commands.insert(work);
    }

    fn deactivate(&self, commands: &mut EntityCommands) {
        commands.remove::<WorkingOnErrand<T>>();
    }

    fn fail_on(&self) -> &Vec<FailureCondition> {
        &self.fail_on
    }

    fn add_failure_condition(&mut self, condition: FailureCondition) {
        self.fail_on.push(condition);
    }
}

#[derive(Debug)]
struct AvailableErrand {
    entity: Entity,
    work_info: Arc<RwLock<AvailableErrandWorkInfo>>,
    factory: Box<dyn ErrandFromAvailableErrand>,
}

#[derive(Debug, Default)]
struct AvailableErrandWorkInfo {
    reserved_by: Weak<ReservedErrand>,
    on_cancel: Vec<Weak<AtomicBool>>,
}

impl AvailableErrand {
    fn new<T: Errand>(entity: Entity, errand: T) -> Self {
        Self {
            entity,
            work_info: default(),
            factory: Box::new(ErrandFromAvailableErrandImpl { value: errand }),
        }
    }

    fn enqueue(&self, worker: Entity, queue: &mut ErrandQueue) -> bool {
        match self.work_info.try_write() {
            Ok(mut lock) => {
                if lock.reserved_by.strong_count() != 0 {
                    return false;
                }

                let reservation = Arc::new(ReservedErrand {
                    reserved_by: worker,
                });

                lock.reserved_by = Arc::downgrade(&reservation);

                let cancelled = Arc::new(AtomicBool::new(false));

                lock.on_cancel.push(Arc::downgrade(&cancelled));

                let fail_on = vec![
                    FailureCondition::TargetErrandCancelled(cancelled),
                    FailureCondition::TargetRemoved(self.entity),
                ];

                self.factory.enqueue(fail_on, reservation, queue);

                true
            }
            Err(TryLockError::WouldBlock) => false,
            Err(TryLockError::Poisoned(underlying)) => {
                error!("Poisoned lock: {:?}", underlying);

                false
            }
        }
    }

    fn errand_type_id(&self) -> TypeId {
        self.factory.errand_type_id()
    }
}

impl Drop for AvailableErrand {
    fn drop(&mut self) {
        if let Ok(mut lock) = self.work_info.write() {
            for cancel in lock.on_cancel.drain(..) {
                if let Some(cancel) = cancel.upgrade() {
                    cancel.store(false, Ordering::Relaxed);
                }
            }
        } else {
            error!("Poisoned lock when dropping available errand");
        }
    }
}

struct ReservedErrand {
    reserved_by: Entity,
}

trait ErrandFromAvailableErrand: Debug + Send + Sync + 'static {
    fn enqueue(
        &self,
        fail_on: Vec<FailureCondition>,
        reservation: Arc<ReservedErrand>,
        queue: &mut ErrandQueue,
    );

    fn errand_type_id(&self) -> TypeId;
}

#[derive(Debug, Clone)]
struct ErrandFromAvailableErrandImpl<T: Errand> {
    value: T,
}

impl<T: Errand> ErrandFromAvailableErrand for ErrandFromAvailableErrandImpl<T> {
    fn enqueue(
        &self,
        fail_on: Vec<FailureCondition>,
        reservation: Arc<ReservedErrand>,
        queue: &mut ErrandQueue,
    ) {
        queue.append_errand(move |errand_id| QueuedErrandImpl {
            id: errand_id,
            errand: self.value.clone(),
            reservation: Some(reservation),
            fail_on,
        });
    }

    fn errand_type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }
}

#[derive(Component, Clone)]
pub struct WorkingOnErrand<T: Errand> {
    id: u64,
    errand: T,
    _reservation: Option<Arc<ReservedErrand>>,
    is_done: bool,
    failed: bool,
}

#[derive(Debug, Clone)]
pub enum FailureCondition {
    TargetRemoved(Entity),
    TargetErrandCancelled(Arc<AtomicBool>),
}

impl<T: Errand> WorkingOnErrand<T> {
    pub fn done(&mut self) {
        self.is_done = true;
    }

    pub fn fail(&mut self) {
        self.failed = true;
    }
}

impl<T: Errand> Deref for WorkingOnErrand<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.errand
    }
}

impl<T: Errand> DerefMut for WorkingOnErrand<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.errand
    }
}

#[derive(Component, Debug)]
pub struct Designation {
    errand: AvailableErrand,
}

impl Designation {
    pub fn new<T: Errand>(entity: Entity, errand: T) -> Self {
        Self {
            errand: AvailableErrand::new(entity, errand),
        }
    }

    pub fn enqueue(&self, worker: Entity, queue: &mut ErrandQueue) -> bool {
        self.errand.enqueue(worker, queue)
    }

    fn errand_type_id(&self) -> TypeId {
        self.errand.errand_type_id()
    }

    pub fn is_errand<E: Errand>(&self) -> bool {
        self.errand_type_id() == TypeId::of::<E>()
    }
}

#[derive(Component)]
struct IsWorking;

fn clear_finished_errands<T: Errand>(
    mut q: Query<(Entity, &WorkingOnErrand<T>, &mut ErrandQueue)>,
    mut commands: Commands,
) {
    for (entity, working_on_errand, mut queue) in q.iter_mut() {
        if working_on_errand.is_done {
            info!("Errand {:?} done", working_on_errand.id);
            commands.entity(entity).remove::<WorkingOnErrand<T>>();
            let first = queue.errands.front();
            if let Some(first) = first {
                if first.id() == working_on_errand.id {
                    queue.errands.pop_front();
                }
            }
        }
    }
}

fn check_failed_errands(mut q: Query<&mut ErrandQueue>, errand_targets: Query<()>) {
    for mut queue in q.iter_mut() {
        queue.errands.retain(|e| {
            for failure_condition in e.fail_on() {
                let retain = match failure_condition {
                    FailureCondition::TargetRemoved(target) =>{
                        if !errand_targets.contains(*target) {
                            info!("Errand target removed");
                            false
                        } else {
                            true
                        }


                    },
                    FailureCondition::TargetErrandCancelled(cancelled) => {
                        if cancelled.load(Ordering::Relaxed) {
                            info!("Errand explicitly cancelled");
                            false
                        } else {
                            true
                        }
                    }
                };

                if !retain {
                    info!("Errand {:?} failed", e.id());
                    return false;
                }
            }

            true
        });
    }
}

fn cancel_current_task_when_overwritten<T: Errand>(
    q: Query<(&ErrandQueue, &WorkingOnErrand<T>, Entity)>,
    mut commands: Commands,
) {
    for (queue, errand, entity) in q.iter() {
        let first = queue.errands.front();

        if let Some(first) = first {
            if first.id() != errand.id {
                info!("Errand {:?} overwritten, removing", errand.id);
                commands.entity(entity).remove::<WorkingOnErrand<T>>();
            }
        } else {
            info!("Working on errand but queue is empty");
            commands.entity(entity).remove::<WorkingOnErrand<T>>();
        }
    }
}

fn has_workers<T: Errand>(q: Query<&WorkingOnErrand<T>>) -> bool {
    !q.is_empty()
}

fn start_next_task<T: Errand>(
    q: Query<&ErrandQueue>,
    mut commands: Commands,
    mut finished_current: RemovedComponents<WorkingOnErrand<T>>,
) {
    for entity in finished_current.iter() {
        if let Ok(queue) = q.get(entity) {
            let first = queue.errands.front();

            let mut entity_commands = commands.entity(entity);
            if let Some(first) = first {
                info!("Activating next errand");
                first.activate(&mut entity_commands);
                entity_commands.insert(IsWorking);
            } else {
                entity_commands.remove::<IsWorking>();
            }
        }
    }
}

fn has_finished_work<T: Errand>(finished: RemovedComponents<WorkingOnErrand<T>>) -> bool {
    !finished.is_empty()
}

fn start_next_errand_in_queue(
    workers: Query<(Entity, &ErrandQueue), Without<IsWorking>>,
    mut commands: Commands,
) {
    for (entity, queue) in workers.iter() {
        let first = queue.errands.front();

        if let Some(first) = first {
            info!("Starting next errand in queue");
            let mut entity_commands = commands.entity(entity);
            first.activate(&mut entity_commands);
            entity_commands.insert(IsWorking);
        }
    }
}

pub struct ErrandsV2Plugin;

impl Plugin for ErrandsV2Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (check_failed_errands, assign_available_errands, start_next_errand_in_queue));
    }
}

pub trait ErrandsV2AppExtensions {
    fn add_errand<E: Errand>(&mut self) -> &mut Self;
}

impl ErrandsV2AppExtensions for App {
    fn add_errand<E: Errand>(&mut self) -> &mut Self {
        self.add_systems(Update, clear_finished_errands::<E>.run_if(has_workers::<E>).before(check_failed_errands))
            .add_systems(Update, cancel_current_task_when_overwritten::<E>.run_if(has_workers::<E>))
            .add_systems(PostUpdate,
                start_next_task::<E>
                    .run_if(has_finished_work::<E>),
            )
            .add_systems(Update, add_capability::<E>.run_if(has_added::<E::WorkerComponent>))
            .add_systems(PostUpdate,
                remove_capability::<E>
                    .run_if(has_removed::<E::WorkerComponent>),
            );

        self
    }
}

fn add_capability<E: Errand>(mut workers: Query<&mut WorkerPriorities, Added<E::WorkerComponent>>) {
    for mut priorities in workers.iter_mut() {
        priorities.add_capability::<E>();
    }
}

fn has_added<C: Component>(q: Query<(), Added<C>>) -> bool {
    !q.is_empty()
}

fn remove_capability<E: Errand>(
    mut workers: Query<&mut WorkerPriorities>,
    mut removed: RemovedComponents<E::WorkerComponent>,
) {
    for entity in removed.iter() {
        if let Ok(mut priorities) = workers.get_mut(entity) {
            priorities.remove_capability::<E>();
        }
    }
}

fn has_removed<C: Component>(q: RemovedComponents<C>) -> bool {
    !q.is_empty()
}

fn assign_available_errands(
    designations: Query<(&Designation, &GlobalTransform)>,
    mut workers: Query<
        (
            Entity,
            &mut ErrandQueue,
            &GlobalTransform,
            &WorkerPriorities,
        ),
        Without<IsWorking>,
    >,
) {
    workers
        .par_iter_mut()
        .for_each_mut(|(entity, mut queue, worker_transform, priorities)| {
            if queue.len() > 0 {
                return;
            }

            let worker_position = worker_transform.translation_vec3a();

            let in_order = priorities
                .priorities
                .iter()
                .filter(|p| p.available && p.priority.is_some())
                .sorted_by_key(|p| p.priority.unwrap())
                .map(|p| p.errand_type_id);

            for errand_type_id in in_order {
                let available_designations = designations
                    .iter()
                    .filter(|(des, _)| des.errand_type_id() == errand_type_id)
                    .map(|(designation, transform)| {
                        (
                            designation,
                            transform
                                .translation_vec3a()
                                .distance_squared(worker_position),
                        )
                    })
                    .sorted_by(|(_, a), (_, b)| a.total_cmp(b))
                    .map(|(des, _)| des);

                for designation in available_designations {
                    if designation.enqueue(entity, &mut queue) {
                        info!("Found errand for worker: {:?}", entity);
                        return;
                    }
                }
            }
        })
}

#[derive(Component, Default)]
pub struct WorkerPriorities {
    priorities: Vec<WorkerPriority>,
}

impl WorkerPriorities {
    fn add_capability<E: Errand>(&mut self) {
        let id = TypeId::of::<E>();
        let priority = self.priorities.iter_mut().find(|p| p.errand_type_id == id);

        if let Some(priority) = priority {
            priority.available = true;
        } else {
            let new_priority = WorkerPriority {
                priority: Some(5),
                available: true,
                errand_type_id: id,
                errand_type_order: E::get_errand_type_order(),
            };

            self.priorities.push(new_priority);
            self.priorities.sort_by_key(|p| p.errand_type_order);
        }
    }

    fn remove_capability<E: Errand>(&mut self) {
        let id = TypeId::of::<E>();
        let priority = self.priorities.iter_mut().find(|p| p.errand_type_id == id);

        if let Some(priority) = priority {
            priority.available = false;
        }
    }
}

pub struct WorkerPriority {
    priority: Option<u8>,
    errand_type_id: TypeId,
    available: bool,
    errand_type_order: i32,
}
