use crate::prelude::*;
use bevy_ecs::system::EntityCommands;
use std::any::Any;
use std::collections::VecDeque;

#[derive(Component)]
pub struct ErrandQueue {
    errands: VecDeque<Box<dyn DowncastableQueuedErrand>>,
}

impl ErrandQueue {
    pub fn new() -> Self {
        Self {
            errands: VecDeque::new(),
        }
    }

    pub fn add_errand(&mut self, errand: impl Component + Errand) {
        self.errands
            .push_back(Box::new(QueuedComponentErrand::new(errand)));
    }

    pub fn override_errand(&mut self, errand: impl Component + Errand) {
        self.clear();
        self.add_errand(errand);
    }

    pub fn prepend_errand(&mut self, errand: impl Component + Errand) {
        self.errands
            .push_front(Box::new(QueuedComponentErrand::new(errand)));
    }

    fn clear(&mut self) {
        self.errands.clear();
    }
}

pub trait ErrandQueueable: Sized {
    fn add_to_errand_queue(self, queue: &mut ErrandQueue);

    fn override_errand(self, queue: &mut ErrandQueue) {
        queue.clear();
        self.add_to_errand_queue(queue);
    }

    fn prepend_to_errand_queue(self, queue: &mut ErrandQueue);
}

macro_rules! invoke_prepend {
    ($queue:ident, $t:ident) => {
        $queue.prepend_errand($t);
    };
    ($queue:ident, $t:ident, $($ts:ident),+) => {
        invoke_prepend!($queue, $($ts),+);
        invoke_prepend!($queue, $t);
    };
}

macro_rules! impl_errand_queueable_for_tuple {
    ($($t:ident),*) => {
        #[allow(non_snake_case)]
        impl<$($t: Errand),*> ErrandQueueable for ($($t,)*) {
            fn add_to_errand_queue(self, queue: &mut ErrandQueue) {
                let ($($t,)*) = self;
                $(queue.add_errand($t);)*
            }

            fn prepend_to_errand_queue(self, queue: &mut ErrandQueue) {
                let ($($t,)*) = self;
                invoke_prepend!(queue, $($t),*);
            }
        }
    }
}

macro_rules! for_each_tuple_ {
    ( $m:ident !! ) => ();
    ( $m:ident !! $h:ident, $($t:ident,)* ) => (
        $m! { $h $(, $t)* }
        for_each_tuple_! { $m !! $($t,)* }
    );
}
macro_rules! for_each_tuple {
    ( $m:ident ) => (
        for_each_tuple_! { $m !! A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, }
    );
}
for_each_tuple!(impl_errand_queueable_for_tuple);

#[cfg(test)]
mod test {
    use super::*;
    use crate::prelude::Component;

    #[derive(Component)]
    struct TestComponent(usize);

    impl Errand for TestComponent {
        fn name(&self) -> String {
            panic!("Not implemented")
        }
    }

    fn assert_errand_value(queue: &ErrandQueue, index: usize, expected_value: usize) {
        let i = (&*queue.errands[index]).as_any();
        let i = i
            .downcast_ref::<QueuedComponentErrand<TestComponent>>()
            .unwrap();
        assert_eq!(i.0.as_ref().unwrap().0, expected_value);
    }

    #[test]
    fn adds_multiple_items_in_correct_order_2() {
        let mut queue = ErrandQueue::new();
        (TestComponent(1), TestComponent(2)).add_to_errand_queue(&mut queue);

        assert_eq!(queue.errands.len(), 2);
        assert_errand_value(&queue, 0, 1);
        assert_errand_value(&queue, 1, 2);
    }

    #[test]
    fn adds_multiple_items_in_correct_order_3() {
        let mut queue = ErrandQueue::new();
        (TestComponent(1), TestComponent(2), TestComponent(3)).add_to_errand_queue(&mut queue);

        assert_eq!(queue.errands.len(), 3);
        assert_errand_value(&queue, 0, 1);
        assert_errand_value(&queue, 1, 2);
        assert_errand_value(&queue, 2, 3);
    }

    #[test]
    fn prepends_multiple_items_in_correct_order_2() {
        let mut queue = ErrandQueue::new();
        (TestComponent(1), TestComponent(2)).prepend_to_errand_queue(&mut queue);

        assert_eq!(queue.errands.len(), 2);
        assert_errand_value(&queue, 0, 1);
        assert_errand_value(&queue, 1, 2);
    }

    #[test]
    fn prepends_multiple_items_in_correct_order_2_multiple_calls() {
        let mut queue = ErrandQueue::new();
        (TestComponent(1), TestComponent(2)).prepend_to_errand_queue(&mut queue);
        (TestComponent(3), TestComponent(4)).prepend_to_errand_queue(&mut queue);

        assert_eq!(queue.errands.len(), 4);
        assert_errand_value(&queue, 0, 3);
        assert_errand_value(&queue, 1, 4);
        assert_errand_value(&queue, 2, 1);
        assert_errand_value(&queue, 3, 2);
    }

    #[test]
    fn prepends_multiple_items_in_correct_order_3() {
        let mut queue = ErrandQueue::new();
        (TestComponent(1), TestComponent(2), TestComponent(3)).prepend_to_errand_queue(&mut queue);

        assert_eq!(queue.errands.len(), 3);
        assert_errand_value(&queue, 0, 1);
        assert_errand_value(&queue, 1, 2);
        assert_errand_value(&queue, 2, 3);
    }
}

impl<T: Errand> ErrandQueueable for T {
    fn add_to_errand_queue(self, queue: &mut ErrandQueue) {
        queue.add_errand(self)
    }

    fn prepend_to_errand_queue(self, queue: &mut ErrandQueue) {
        queue.prepend_errand(self);
    }
}

pub trait QueuedErrand: Sync + Send + 'static {
    fn insert_component(&mut self, commands: &mut EntityCommands);
}

trait DowncastableQueuedErrand: QueuedErrand + Any {
    fn as_any(&self) -> &dyn Any;
}

impl<T: QueuedErrand + Any> DowncastableQueuedErrand for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct QueuedComponentErrand<T: Component + Errand>(Option<T>);

impl<T: Component + Errand> QueuedComponentErrand<T> {
    pub fn new(comp: T) -> Self {
        Self(Some(comp))
    }
}

impl<T: Component + Errand> QueuedErrand for QueuedComponentErrand<T> {
    fn insert_component(&mut self, commands: &mut EntityCommands) {
        if let Some(comp) = self.0.take() {
            commands.insert(comp);
        }
    }
}

fn on_errand_finished_system<T: Component>(
    mut finished: RemovedComponents<T>,
    mut commands: Commands,
    mut errand_queues: Query<&mut ErrandQueue>,
) {
    for entity in finished.iter() {
        if let Ok(mut errand_queue) = errand_queues.get_mut(entity) {
            let mut entity_commands = commands.entity(entity);
            if let Some(mut errand) = errand_queue.errands.pop_front() {
                errand.insert_component(&mut entity_commands);
            } else {
                entity_commands.remove::<WorkingOnErrand>();
            }
        } else {
            warn!("Entity {:?} finished errand, but has no errand queue", entity);
        }
    }
}

fn has_finished_errand<T: Component>(finished: RemovedComponents<T>) -> bool {
    finished.len() > 0
}

pub trait ErrandQueuePluginExtensions {
    fn register_errand<T: Component + Errand>(&mut self) -> &mut Self;
}

impl ErrandQueuePluginExtensions for App {
    fn register_errand<T: Component + Errand>(&mut self) -> &mut Self {
        self.add_system(on_errand_finished_system::<T>.run_if(has_finished_errand::<T>))
    }
}

#[derive(Component)]
pub struct WorkingOnErrand;

pub fn dequeue_next_errand(
    mut workers: Query<(Entity, &mut ErrandQueue), Without<WorkingOnErrand>>,
    mut commands: Commands,
) {
    for (entity, mut queue) in workers.iter_mut() {
        if let Some(mut next) = queue.errands.pop_front() {
            let mut entity_commands = commands.entity(entity);
            next.insert_component(&mut entity_commands);
            entity_commands.insert(WorkingOnErrand);
        }
    }
}

pub trait Errand: Component {
    fn name(&self) -> String;
}
