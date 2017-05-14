use time::precise_time_ns;

// FPS is smoothed with an exponentially-weighted moving average.
// This is the proportion of the old FPS to keep on each step.
const SMOOTHING: f32
    = 0.9;

// How often to output FPS, in milliseconds.
const OUTPUT_INTERVAL: u64
    = 5_000;

pub struct FPSTracker {
    last_frame_time: u64,
    last_fps_output_time: u64,
    smoothed_fps: f32,
}

impl FPSTracker {
    pub fn new() -> FPSTracker {
        let t = precise_time_ns();
        FPSTracker {
            last_frame_time: t,
            last_fps_output_time: t,
            smoothed_fps: 0.0,
        }
    }

    pub fn tick(&mut self) {
        let this_frame_time = precise_time_ns();
        let instant_fps = 1e9 / ((this_frame_time - self.last_frame_time) as f32);
        self.smoothed_fps = SMOOTHING * self.smoothed_fps
                     + (1.0-SMOOTHING) * instant_fps;
        self.last_frame_time = this_frame_time;

        if (this_frame_time - self.last_fps_output_time)
            >= 1_000_000 * OUTPUT_INTERVAL
        {
            println!("Frames per second: {:7.2}", self.smoothed_fps);
            self.last_fps_output_time = this_frame_time;
        }
    }
}
