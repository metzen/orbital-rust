use bevy::camera::visibility::Layer;

pub trait LayerExt {
    const FOREGROUND: Layer;
    const ORBIT: Layer;
    const BACKGROUND: Layer;
    const MAIN: Layer;
}

impl LayerExt for Layer {
    const FOREGROUND: Layer = 0;
    const ORBIT: Layer = 1;
    const BACKGROUND: Layer = 2;
    const MAIN: Layer = 3;
}
