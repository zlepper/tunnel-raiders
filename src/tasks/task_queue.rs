use crate::prelude::*;
use bevy_ecs::system::EntityCommands;
use std::any::Any;
use std::collections::VecDeque;

#[derive(Component)]
pub struct TaskQueue {
    tasks: VecDeque<Box<dyn DowncastableQueuedTask>>,
}

impl TaskQueue {
    pub fn new() -> Self {
        Self {
            tasks: VecDeque::new(),
        }
    }

    pub fn add_task(&mut self, task: impl Component + Task) {
        self.tasks
            .push_back(Box::new(QueuedComponentTask::new(task)));
    }

    pub fn override_task(&mut self, task: impl Component + Task) {
        self.clear();
        self.add_task(task);
    }

    pub fn prepend_task(&mut self, task: impl Component + Task) {
        self.tasks
            .push_front(Box::new(QueuedComponentTask::new(task)));
    }

    fn clear(&mut self) {
        self.tasks.clear();
    }
}

pub trait TaskQueueable: Sized {
    fn add_to_task_queue(self, queue: &mut TaskQueue);

    fn override_task(self, queue: &mut TaskQueue) {
        queue.clear();
        self.add_to_task_queue(queue);
    }

    fn prepend_to_task_queue(self, queue: &mut TaskQueue);
}

macro_rules! invoke_prepend {
    ($queue:ident, $t:ident) => {
        $queue.prepend_task($t);
    };
    ($queue:ident, $t:ident, $($ts:ident),+) => {
        invoke_prepend!($queue, $($ts),+);
        invoke_prepend!($queue, $t);
    };
}

macro_rules! impl_task_queueable_for_tuple {
    ($($t:ident),*) => {
        #[allow(non_snake_case)]
        impl<$($t: Task),*> TaskQueueable for ($($t,)*) {
            fn add_to_task_queue(self, queue: &mut TaskQueue) {
                let ($($t,)*) = self;
                $(queue.add_task($t);)*
            }

            fn prepend_to_task_queue(self, queue: &mut TaskQueue) {
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
for_each_tuple!(impl_task_queueable_for_tuple);

#[cfg(test)]
mod test {
    use super::*;
    use crate::prelude::Component;

    #[derive(Component)]
    struct TestComponent(usize);

    impl Task for TestComponent {
        fn name(&self) -> String {
            panic!("Not implemented")
        }
    }

    fn assert_task_value(queue: &TaskQueue, index: usize, expected_value: usize) {
        let i = (&*queue.tasks[index]).as_any();
        let i = i
            .downcast_ref::<QueuedComponentTask<TestComponent>>()
            .unwrap();
        assert_eq!(i.0.as_ref().unwrap().0, expected_value);
    }

    #[test]
    fn adds_multiple_items_in_correct_order_2() {
        let mut queue = TaskQueue::new();
        (TestComponent(1), TestComponent(2)).add_to_task_queue(&mut queue);

        assert_eq!(queue.tasks.len(), 2);
        assert_task_value(&queue, 0, 1);
        assert_task_value(&queue, 1, 2);
    }

    #[test]
    fn adds_multiple_items_in_correct_order_3() {
        let mut queue = TaskQueue::new();
        (TestComponent(1), TestComponent(2), TestComponent(3)).add_to_task_queue(&mut queue);

        assert_eq!(queue.tasks.len(), 3);
        assert_task_value(&queue, 0, 1);
        assert_task_value(&queue, 1, 2);
        assert_task_value(&queue, 2, 3);
    }

    #[test]
    fn prepends_multiple_items_in_correct_order_2() {
        let mut queue = TaskQueue::new();
        (TestComponent(1), TestComponent(2)).prepend_to_task_queue(&mut queue);

        assert_eq!(queue.tasks.len(), 2);
        assert_task_value(&queue, 0, 1);
        assert_task_value(&queue, 1, 2);
    }

    #[test]
    fn prepends_multiple_items_in_correct_order_2_multiple_calls() {
        let mut queue = TaskQueue::new();
        (TestComponent(1), TestComponent(2)).prepend_to_task_queue(&mut queue);
        (TestComponent(3), TestComponent(4)).prepend_to_task_queue(&mut queue);

        assert_eq!(queue.tasks.len(), 4);
        assert_task_value(&queue, 0, 3);
        assert_task_value(&queue, 1, 4);
        assert_task_value(&queue, 2, 1);
        assert_task_value(&queue, 3, 2);
    }

    #[test]
    fn prepends_multiple_items_in_correct_order_3() {
        let mut queue = TaskQueue::new();
        (TestComponent(1), TestComponent(2), TestComponent(3)).prepend_to_task_queue(&mut queue);

        assert_eq!(queue.tasks.len(), 3);
        assert_task_value(&queue, 0, 1);
        assert_task_value(&queue, 1, 2);
        assert_task_value(&queue, 2, 3);
    }
}

impl<T: Task> TaskQueueable for T {
    fn add_to_task_queue(self, queue: &mut TaskQueue) {
        queue.add_task(self)
    }

    fn prepend_to_task_queue(self, queue: &mut TaskQueue) {
        queue.prepend_task(self);
    }
}

pub trait QueuedTask: Sync + Send + 'static {
    fn insert_component(&mut self, commands: &mut EntityCommands);
}

trait DowncastableQueuedTask: QueuedTask + Any {
    fn as_any(&self) -> &dyn Any;
}

impl<T: QueuedTask + Any> DowncastableQueuedTask for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct QueuedComponentTask<T: Component + Task>(Option<T>);

impl<T: Component + Task> QueuedComponentTask<T> {
    pub fn new(comp: T) -> Self {
        Self(Some(comp))
    }
}

impl<T: Component + Task> QueuedTask for QueuedComponentTask<T> {
    fn insert_component(&mut self, commands: &mut EntityCommands) {
        if let Some(comp) = self.0.take() {
            commands.insert(comp);
        }
    }
}

fn on_task_finished_system<T: Component>(
    mut finished: RemovedComponents<T>,
    mut commands: Commands,
    mut task_queues: Query<&mut TaskQueue>,
) {
    for entity in finished.iter() {
        if let Ok(mut task_queue) = task_queues.get_mut(entity) {
            let mut entity_commands = commands.entity(entity);
            if let Some(mut task) = task_queue.tasks.pop_front() {
                task.insert_component(&mut entity_commands);
            } else {
                entity_commands.remove::<WorkingOnTask>();
            }
        } else {
            warn!("Entity {:?} finished task, but has no task queue", entity);
        }
    }
}

fn has_finished_task<T: Component>(finished: RemovedComponents<T>) -> bool {
    finished.len() > 0
}

pub trait TaskQueuePluginExtensions {
    fn register_task<T: Component + Task>(&mut self) -> &mut Self;
}

impl TaskQueuePluginExtensions for App {
    fn register_task<T: Component + Task>(&mut self) -> &mut Self {
        self.add_system(on_task_finished_system::<T>.run_if(has_finished_task::<T>))
    }
}

#[derive(Component)]
pub struct WorkingOnTask;

pub fn dequeue_next_task(
    mut workers: Query<(Entity, &mut TaskQueue), Without<WorkingOnTask>>,
    mut commands: Commands,
) {
    for (entity, mut queue) in workers.iter_mut() {
        if let Some(mut next) = queue.tasks.pop_front() {
            let mut entity_commands = commands.entity(entity);
            next.insert_component(&mut entity_commands);
            entity_commands.insert(WorkingOnTask);
        }
    }
}

pub trait Task: Component {
    fn name(&self) -> String;
}
