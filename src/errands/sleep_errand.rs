use crate::errands::WorkingOnErrand;
use crate::prelude::*;

#[derive(Clone, Debug)]
pub struct SleepErrand {
    pub duration: f32,
}

impl Errand for SleepErrand {
    type WorkerComponent = ErrandQueue;

    fn get_errand_type_order() -> i32 {
        1000
    }
}

pub fn execute_sleep_errand(
    mut query: Query<&mut WorkingOnErrand<SleepErrand>>,
    time: Res<Time>,
) {
    for mut errand in query.iter_mut() {
        errand.duration -= time.delta_seconds();
        if errand.duration <= 0.0 {
            errand.done();
        }
    }
}
