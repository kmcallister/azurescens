tunapanel! {
    #[title = "Azurescens"]
    pub struct Params {
        #[label = "Invert each frame"]
        invert: bool = true,

        #[label = "Fade (non-inverting mode)"]
        fade: f32 = 0.9,

        #[label = "Permute color channels"]
        permute_colors: bool = true,

        #[label = "Color cycle rate"]
        color_cycle_rate: f32 = 1.0,

        #[label = "Mix for linear interpolation"]
        mix_linear: f32 = 0.0,

        #[label = "Time varying mix for linear"]
        mix_linear_tv: f32 = 0.2,
    }
}
